//! Context window bloat, lockfile detector and large artifact truncator.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Alert triggered when context exceeds healthy token budgets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextAlert {
    pub message: String,
    pub approximate_tokens: u64,
    pub severity: &'static str,
}

/// Statistics returned after sanitizing a prompt payload.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruncationStats {
    pub count_truncated: usize,
    pub bytes_saved: usize,
    pub estimated_tokens_saved: u64,
}

pub struct ContextWindowGuard {
    pub warning_threshold_tokens: u64,
    pub critical_threshold_tokens: u64,
    max_artifact_chars: usize,
}

static LOCKFILE_REGEX: OnceLock<Regex> = OnceLock::new();
static BASE64_REGEX: OnceLock<Regex> = OnceLock::new();

impl ContextWindowGuard {
    pub fn new(warning_threshold: u64, critical_threshold: u64) -> Self {
        Self {
            warning_threshold_tokens: warning_threshold,
            critical_threshold_tokens: critical_threshold,
            max_artifact_chars: 4_000, // 4KB artifact limit before truncation kicks in
        }
    }

    pub fn with_max_artifact_size(mut self, max_chars: usize) -> Self {
        self.max_artifact_chars = max_chars;
        self
    }

    /// Inspect context payload size and flag dangerous token bloat.
    pub fn inspect_payload(&self, payload_bytes: &[u8]) -> Option<ContextAlert> {
        let approx_tokens = (payload_bytes.len() / 4) as u64;

        if approx_tokens >= self.critical_threshold_tokens {
            Some(ContextAlert {
                message: format!(
                    "CRITICAL context explosion: ~{}k tokens loaded (limit: {}k)",
                    approx_tokens / 1000,
                    self.critical_threshold_tokens / 1000
                ),
                approximate_tokens: approx_tokens,
                severity: "CRITICAL",
            })
        } else if approx_tokens >= self.warning_threshold_tokens {
            Some(ContextAlert {
                message: format!(
                    "WARNING high context footprint: ~{}k tokens loaded",
                    approx_tokens / 1000
                ),
                approximate_tokens: approx_tokens,
                severity: "WARNING",
            })
        } else {
            None
        }
    }

    /// Check if a text block appears to be an uncompressed lockfile or data dump.
    fn is_dangerous_artifact(text: &str) -> Option<&'static str> {
        if text.len() < 2_000 {
            return None;
        }

        // 1. Lockfiles: package-lock.json, Cargo.lock, yarn.lock, pnpm-lock
        let lockfile_re = LOCKFILE_REGEX.get_or_init(|| {
            Regex::new(r#"(?:"lockfileVersion"\s*:|\[\[package\]\]\s*\n\s*name\s*=)"#).unwrap()
        });
        if lockfile_re.is_match(text) {
            return Some("Lockfile (package-lock/Cargo.lock)");
        }

        // 2. Giant base64 data URIs or inline binary dumps
        let base64_re = BASE64_REGEX.get_or_init(|| {
            Regex::new(r#"data:image/[a-zA-Z]+;base64,[A-Za-z0-9+/=]{1000,}"#).unwrap()
        });
        if base64_re.is_match(text) {
            return Some("Inline Base64 Asset Dump");
        }

        // 3. Minified JS/JSON blob (huge single lines without whitespace)
        if text.len() > 16_000 && !text.contains('\n') {
            return Some("Minified Code/Data Blob");
        }

        None
    }

    /// Truncate an oversized artifact into a safe head/tail summary.
    pub fn truncate_artifact(&self, text: &str) -> (String, bool, usize) {
        if text.len() <= self.max_artifact_chars {
            return (text.to_string(), false, 0);
        }

        if let Some(artifact_type) = Self::is_dangerous_artifact(text) {
            let head_len = 500.min(text.len() / 4);
            let tail_len = 500.min(text.len() / 4);
            let head = &text[..head_len];
            let tail = &text[text.len() - tail_len..];

            let saved_bytes = text.len() - (head_len + tail_len);
            let saved_tokens = (saved_bytes / 4) as u64;

            let truncated = format!(
                "{}\n\n[TRUNCATED_BY_BURNRATE: {} detected ({} bytes). Truncated to prevent context explosion. Saved ~{} tokens]\n\n{}",
                head, artifact_type, text.len(), saved_tokens, tail
            );

            (truncated, true, saved_bytes)
        } else {
            (text.to_string(), false, 0)
        }
    }

    /// Recursively sanitize JSON payload (e.g. Anthropic/OpenAI messages).
    pub fn sanitize_json(&self, val: &mut serde_json::Value) -> TruncationStats {
        let mut stats = TruncationStats::default();
        self.traverse_and_truncate(val, &mut stats);
        stats
    }

    fn traverse_and_truncate(&self, val: &mut serde_json::Value, stats: &mut TruncationStats) {
        match val {
            serde_json::Value::String(s) => {
                let (truncated, was_cut, saved) = self.truncate_artifact(s);
                if was_cut {
                    *s = truncated;
                    stats.count_truncated += 1;
                    stats.bytes_saved += saved;
                    stats.estimated_tokens_saved += (saved / 4) as u64;
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.traverse_and_truncate(item, stats);
                }
            }
            serde_json::Value::Object(map) => {
                for (_, v) in map.iter_mut() {
                    self.traverse_and_truncate(v, stats);
                }
            }
            _ => {}
        }
    }
}

impl Default for ContextWindowGuard {
    fn default() -> Self {
        Self::new(30_000, 80_000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_truncation() {
        let guard = ContextWindowGuard::default();
        // Construct simulated 10KB package-lock.json
        let mut lockfile_body = String::from(r#"{"name": "test-app", "lockfileVersion": 3, "packages": {"#);
        for i in 0..200 {
            lockfile_body.push_str(&format!(r#""node_modules/pkg-{}": {{"version": "1.0.{}"}}, "#, i, i));
        }
        lockfile_body.push_str(r#""end": true}}"#);

        assert!(lockfile_body.len() > 6_000);
        let (sanitized, was_cut, saved) = guard.truncate_artifact(&lockfile_body);
        assert!(was_cut);
        assert!(saved > 4_000);
        assert!(sanitized.contains("[TRUNCATED_BY_BURNRATE: Lockfile (package-lock/Cargo.lock)"));

        let mut json = serde_json::json!({
            "messages": [
                { "role": "user", "content": lockfile_body }
            ]
        });

        let stats = guard.sanitize_json(&mut json);
        assert_eq!(stats.count_truncated, 1);
        assert!(stats.estimated_tokens_saved > 1_000);
    }
}
