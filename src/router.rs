//! Token-cost optimal model cascade router for multi-agent workflows.

use crate::pricing::PricingRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub selected_model: String,
    pub estimated_cost_usd: f64,
    pub reasoning: String,
    pub downgraded: bool,
}

pub struct ModelCostRouter {
    pub budget_threshold_remaining: f64,
}

impl ModelCostRouter {
    pub fn new(budget_threshold_remaining: f64) -> Self {
        Self {
            budget_threshold_remaining,
        }
    }

    /// Select optimal model based on task complexity and remaining budget.
    pub fn route_task(&self, task_prompt: &str, remaining_budget_usd: f64) -> RouteDecision {
        let is_complex = task_prompt.contains("refactor")
            || task_prompt.contains("architecture")
            || task_prompt.contains("debug complex")
            || task_prompt.len() > 1000;

        let approx_tokens = (task_prompt.len() / 4) as u64;

        if remaining_budget_usd < self.budget_threshold_remaining {
            // Budget critical: downgrade to lightweight high-throughput model
            let pricing = PricingRegistry::lookup("claude-haiku");
            let cost = pricing.calculate_cost(approx_tokens, 200, 0, 0);
            return RouteDecision {
                selected_model: "claude-3-5-haiku-20241022".to_string(),
                estimated_cost_usd: cost,
                reasoning: "Budget threshold low: downgraded to Claude 3.5 Haiku to preserve funds".to_string(),
                downgraded: true,
            };
        }

        if is_complex {
            let pricing = PricingRegistry::lookup("claude-sonnet");
            let cost = pricing.calculate_cost(approx_tokens, 600, 0, 0);
            RouteDecision {
                selected_model: "claude-3-7-sonnet-20250219".to_string(),
                estimated_cost_usd: cost,
                reasoning: "Complex reasoning detected: routed to Claude 3.7 Sonnet".to_string(),
                downgraded: false,
            }
        } else {
            let pricing = PricingRegistry::lookup("claude-haiku");
            let cost = pricing.calculate_cost(approx_tokens, 150, 0, 0);
            RouteDecision {
                selected_model: "claude-3-5-haiku-20241022".to_string(),
                estimated_cost_usd: cost,
                reasoning: "Standard task: routed to cost-effective Claude 3.5 Haiku".to_string(),
                downgraded: false,
            }
        }
    }
}
