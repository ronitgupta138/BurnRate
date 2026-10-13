//! Local offline model fallback router to prevent catastrophic session aborts.

use serde::{Deserialize, Serialize};

/// Decision returned when evaluating whether to fall back to a local model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FallbackDecision {
    pub is_fallback: bool,
    pub target_endpoint: String,
    pub target_model: String,
    pub reason: String,
}

pub struct LocalFallbackEngine {
    pub fallback_threshold_ratio: f64,
    pub local_endpoint: String,
    pub default_local_model: String,
}

impl LocalFallbackEngine {
    pub fn new() -> Self {
        Self {
            fallback_threshold_ratio: 0.95, // 95% spend threshold
            local_endpoint: "http://localhost:11434/v1".to_string(),
            default_local_model: "deepseek-r1:latest".to_string(),
        }
    }

    pub fn with_settings(
        threshold_ratio: f64,
        local_endpoint: impl Into<String>,
        default_local_model: impl Into<String>,
    ) -> Self {
        Self {
            fallback_threshold_ratio: threshold_ratio,
            local_endpoint: local_endpoint.into(),
            default_local_model: default_local_model.into(),
        }
    }

    /// Evaluate if the current session spend warrants switching to zero-cost local LLMs.
    pub fn evaluate(
        &self,
        current_spent_usd: f64,
        max_spend_usd: f64,
        requested_model: &str,
    ) -> FallbackDecision {
        if max_spend_usd > 0.0 && (current_spent_usd / max_spend_usd) >= self.fallback_threshold_ratio {
            let pct = (current_spent_usd / max_spend_usd) * 100.0;
            FallbackDecision {
                is_fallback: true,
                target_endpoint: self.local_endpoint.clone(),
                target_model: self.default_local_model.clone(),
                reason: format!(
                    "Budget critical ({:.1}% consumed, ${:.2} / ${:.2}): routed to local $0 offline model (`{}`) to prevent session termination",
                    pct, current_spent_usd, max_spend_usd, self.default_local_model
                ),
            }
        } else {
            FallbackDecision {
                is_fallback: false,
                target_endpoint: "upstream".to_string(),
                target_model: requested_model.to_string(),
                reason: "Budget healthy: standard upstream routing maintained".to_string(),
            }
        }
    }
}

impl Default for LocalFallbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_trigger_at_95_percent_spend() {
        let engine = LocalFallbackEngine::new();

        // 1. Healthy spend at $0.80 / $2.00 (40%) -> No fallback
        let dec1 = engine.evaluate(0.80, 2.00, "claude-3-7-sonnet");
        assert!(!dec1.is_fallback);
        assert_eq!(dec1.target_model, "claude-3-7-sonnet");

        // 2. Critical spend at $1.92 / $2.00 (96%) -> Fallback triggered
        let dec2 = engine.evaluate(1.92, 2.00, "claude-3-7-sonnet");
        assert!(dec2.is_fallback);
        assert_eq!(dec2.target_model, "deepseek-r1:latest");
        assert!(dec2.reason.contains("routed to local $0 offline model"));
    }
}
