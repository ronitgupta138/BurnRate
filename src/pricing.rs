//! Frontier model pricing tables with prompt cache read/write discounts.

use serde::{Deserialize, Serialize};

/// Per-million token pricing rates for frontier models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub model_id: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_read_per_million: f64,
    pub cache_write_per_million: f64,
}

impl ModelPricing {
    pub fn new(
        model_id: impl Into<String>,
        input: f64,
        output: f64,
        cache_read: f64,
        cache_write: f64,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            input_per_million: input,
            output_per_million: output,
            cache_read_per_million: cache_read,
            cache_write_per_million: cache_write,
        }
    }

    /// Calculate exact cost in USD for a given turn.
    pub fn calculate_cost(
        &self,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        let cache_read_cost = (cache_read_tokens as f64 / 1_000_000.0) * self.cache_read_per_million;
        let cache_write_cost = (cache_write_tokens as f64 / 1_000_000.0) * self.cache_write_per_million;

        input_cost + output_cost + cache_read_cost + cache_write_cost
    }
}

/// Registry of known models.
pub struct PricingRegistry;

impl PricingRegistry {
    /// Look up pricing table for model name (fuzzy match).
    pub fn lookup(model: &str) -> ModelPricing {
        let m = model.to_lowercase();
        if m.contains("opus") {
            // Claude 3.5 / 4.5 Opus ($15/$75)
            ModelPricing::new("claude-opus", 15.0, 75.0, 1.5, 18.75)
        } else if m.contains("sonnet") {
            // Claude 3.5 / 3.7 Sonnet ($3/$15)
            ModelPricing::new("claude-sonnet", 3.0, 15.0, 0.30, 3.75)
        } else if m.contains("haiku") {
            // Claude 3.5 Haiku ($0.80/$4)
            ModelPricing::new("claude-haiku", 0.80, 4.0, 0.08, 1.0)
        } else if m.contains("gpt-4o-mini") {
            // GPT-4o-mini ($0.15/$0.60)
            ModelPricing::new("gpt-4o-mini", 0.15, 0.60, 0.075, 0.15)
        } else if m.contains("gpt-4o") || m.contains("gpt-4") {
            // GPT-4o ($2.50/$10)
            ModelPricing::new("gpt-4o", 2.50, 10.0, 1.25, 2.50)
        } else if m.contains("deepseek") {
            // DeepSeek-V3 ($0.14/$0.28)
            ModelPricing::new("deepseek-v3", 0.14, 0.28, 0.014, 0.14)
        } else {
            // Conservative fallback default (Sonnet tier)
            ModelPricing::new("standard-agent", 3.0, 15.0, 0.30, 3.75)
        }
    }
}
