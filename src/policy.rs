//! Declarative budget and security policy governance engine.

use serde::{Deserialize, Serialize};

/// Verdict returned when evaluating a request against the active policy suite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyVerdict {
    Allowed,
    Warn(String),
    Denied(String),
}

/// A single declarative governance rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyRule {
    /// Deny turns where input token payload exceeds this limit.
    MaxInputTokens(u64),
    /// Disallow expensive models if session spend ratio exceeds threshold (e.g. 0.50 = 50%).
    DenyExpensiveModelOnHighSpend {
        model_substring: String,
        spend_ratio_threshold: f64,
    },
    /// Emit warning if a single turn cost exceeds threshold in USD.
    MaxSingleTurnCostAlert(f64),
}

pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn new(rules: Vec<PolicyRule>) -> Self {
        Self { rules }
    }

    /// Add a rule to the active policy engine.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    /// Evaluate a request against all configured rules.
    pub fn evaluate(
        &self,
        model: &str,
        input_tokens: u64,
        estimated_cost: f64,
        current_spent_usd: f64,
        max_spend_usd: f64,
    ) -> PolicyVerdict {
        let spend_ratio = if max_spend_usd > 0.0 {
            current_spent_usd / max_spend_usd
        } else {
            0.0
        };

        for rule in &self.rules {
            match rule {
                PolicyRule::MaxInputTokens(max_tok) => {
                    if input_tokens > *max_tok {
                        return PolicyVerdict::Denied(format!(
                            "Policy Violation: Input tokens ({}) exceeds maximum allowed limit ({})",
                            input_tokens, max_tok
                        ));
                    }
                }
                PolicyRule::DenyExpensiveModelOnHighSpend {
                    model_substring,
                    spend_ratio_threshold,
                } => {
                    if model.contains(model_substring) && spend_ratio >= *spend_ratio_threshold {
                        return PolicyVerdict::Denied(format!(
                            "Policy Violation: Model `{}` blocked because session spend ({:.1}%) exceeds allowed threshold ({:.1}%)",
                            model,
                            spend_ratio * 100.0,
                            spend_ratio_threshold * 100.0
                        ));
                    }
                }
                PolicyRule::MaxSingleTurnCostAlert(threshold) => {
                    if estimated_cost > *threshold {
                        return PolicyVerdict::Warn(format!(
                            "Policy Alert: High turn cost estimated at ${:.4} (threshold: ${:.2})",
                            estimated_cost, threshold
                        ));
                    }
                }
            }
        }

        PolicyVerdict::Allowed
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self {
            rules: vec![
                PolicyRule::MaxInputTokens(150_000),
                PolicyRule::DenyExpensiveModelOnHighSpend {
                    model_substring: "opus".to_string(),
                    spend_ratio_threshold: 0.70, // Block Opus once 70% of budget is consumed
                },
                PolicyRule::MaxSingleTurnCostAlert(0.50),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_engine_rule_enforcement() {
        let engine = PolicyEngine::default();

        // 1. Normal benign turn -> Allowed
        let v1 = engine.evaluate("claude-sonnet", 5_000, 0.05, 0.20, 2.00);
        assert_eq!(v1, PolicyVerdict::Allowed);

        // 2. High turn cost -> Warn
        let v2 = engine.evaluate("claude-sonnet", 5_000, 0.85, 0.20, 2.00);
        match v2 {
            PolicyVerdict::Warn(msg) => assert!(msg.contains("High turn cost")),
            _ => panic!("Expected warning"),
        }

        // 3. Opus when spend ratio is 80% (threshold 70%) -> Denied
        let v3 = engine.evaluate("claude-opus", 5_000, 0.15, 1.60, 2.00);
        match v3 {
            PolicyVerdict::Denied(msg) => assert!(msg.contains("blocked because session spend")),
            _ => panic!("Expected denial"),
        }

        // 4. Input tokens exceed 150k limit -> Denied
        let v4 = engine.evaluate("claude-sonnet", 160_000, 0.10, 0.10, 2.00);
        match v4 {
            PolicyVerdict::Denied(msg) => assert!(msg.contains("Input tokens (160000) exceeds")),
            _ => panic!("Expected denial"),
        }
    }
}
