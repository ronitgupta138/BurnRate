//! Context window bloat and large artifact linter.

/// Alert triggered when context exceeds healthy token budgets.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContextAlert {
    pub message: String,
    pub approximate_tokens: u64,
    pub severity: &'static str,
}

pub struct ContextWindowGuard {
    pub warning_threshold_tokens: u64,
    pub critical_threshold_tokens: u64,
}

impl ContextWindowGuard {
    pub fn new(warning_threshold: u64, critical_threshold: u64) -> Self {
        Self {
            warning_threshold_tokens: warning_threshold,
            critical_threshold_tokens: critical_threshold,
        }
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
}

impl Default for ContextWindowGuard {
    fn default() -> Self {
        Self::new(30_000, 80_000)
    }
}
