//! In-line secret and credential scanner and redactor for prompt payloads.

use regex::{Captures, Regex};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Recognized credential and secret types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SecretType {
    AnthropicKey,
    OpenAiKey,
    AwsAccessKeyId,
    GithubPat,
    PemPrivateKey,
}

impl SecretType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AnthropicKey => "ANTHROPIC_KEY",
            Self::OpenAiKey => "OPENAI_KEY",
            Self::AwsAccessKeyId => "AWS_ACCESS_KEY_ID",
            Self::GithubPat => "GITHUB_PAT",
            Self::PemPrivateKey => "PEM_PRIVATE_KEY",
        }
    }

    pub fn redaction_tag(&self) -> &'static str {
        match self {
            Self::AnthropicKey => "[REDACTED_BY_BURNRATE:ANTHROPIC_KEY]",
            Self::OpenAiKey => "[REDACTED_BY_BURNRATE:OPENAI_KEY]",
            Self::AwsAccessKeyId => "[REDACTED_BY_BURNRATE:AWS_ACCESS_KEY_ID]",
            Self::GithubPat => "[REDACTED_BY_BURNRATE:GITHUB_PAT]",
            Self::PemPrivateKey => "[REDACTED_BY_BURNRATE:PEM_PRIVATE_KEY]",
        }
    }
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Secret guard that redacts sensitive credentials and API tokens.
pub struct SecretGuard {
    regex: Regex,
    total_redacted: AtomicUsize,
}

impl SecretGuard {
    /// Create a new SecretGuard compiled with token pattern matchers.
    pub fn new() -> Self {
        let pattern = concat!(
            "(?P<ANTHROPIC_KEY>sk-ant-[A-Za-z0-9_-]{20,})|",
            "(?P<OPENAI_KEY>sk-proj-[A-Za-z0-9_-]{20,}|sk-[A-Za-z0-9]{20,})|",
            "(?P<AWS_ACCESS_KEY_ID>AKIA[0-9A-Z]{16})|",
            "(?P<GITHUB_PAT>ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{22,})|",
            "(?P<PEM_PRIVATE_KEY>-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----)"
        );
        let regex = Regex::new(pattern).expect("Valid secret regex pattern");
        Self {
            regex,
            total_redacted: AtomicUsize::new(0),
        }
    }

    /// Redact recognized secrets in text, returning the sanitized text and count of secrets redacted.
    pub fn redact(&self, text: &str) -> (String, usize) {
        let mut count = 0;
        let sanitized = self.regex.replace_all(text, |caps: &Captures| {
            count += 1;
            if caps.name("ANTHROPIC_KEY").is_some() {
                "[REDACTED_BY_BURNRATE:ANTHROPIC_KEY]"
            } else if caps.name("OPENAI_KEY").is_some() {
                "[REDACTED_BY_BURNRATE:OPENAI_KEY]"
            } else if caps.name("AWS_ACCESS_KEY_ID").is_some() {
                "[REDACTED_BY_BURNRATE:AWS_ACCESS_KEY_ID]"
            } else if caps.name("GITHUB_PAT").is_some() {
                "[REDACTED_BY_BURNRATE:GITHUB_PAT]"
            } else if caps.name("PEM_PRIVATE_KEY").is_some() {
                "[REDACTED_BY_BURNRATE:PEM_PRIVATE_KEY]"
            } else {
                "[REDACTED_BY_BURNRATE:UNKNOWN]"
            }
        });

        if count > 0 {
            self.total_redacted.fetch_add(count, Ordering::Relaxed);
        }

        (sanitized.into_owned(), count)
    }

    /// Redact secrets in raw bytes.
    pub fn redact_bytes(&self, bytes: &[u8]) -> (Vec<u8>, usize) {
        let text = String::from_utf8_lossy(bytes);
        let (redacted, count) = self.redact(&text);
        if count == 0 {
            (bytes.to_vec(), 0)
        } else {
            (redacted.into_bytes(), count)
        }
    }

    /// Check if text contains any recognized secrets.
    pub fn contains_secrets(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    /// Get total number of secrets redacted across all calls to this guard instance.
    pub fn total_redacted(&self) -> usize {
        self.total_redacted.load(Ordering::Relaxed)
    }
}

impl Default for SecretGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SecretGuard {
    fn clone(&self) -> Self {
        Self {
            regex: self.regex.clone(),
            total_redacted: AtomicUsize::new(self.total_redacted()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_guard_unit() {
        let guard = SecretGuard::new();
        let (redacted, count) = guard.redact("key: AKIAIOSFODNN7EXAMPLE");
        assert_eq!(count, 1);
        assert_eq!(redacted, "key: [REDACTED_BY_BURNRATE:AWS_ACCESS_KEY_ID]");
    }
}
