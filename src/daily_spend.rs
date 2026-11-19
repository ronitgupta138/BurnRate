use std::collections::HashMap;

/// Aggregates and attributes daily token spend across AI model families.
#[derive(Debug, Clone, Default)]
pub struct DailySpendAggregator {
    spend_by_model: HashMap<String, f64>,
}

impl DailySpendAggregator {
    pub fn new() -> Self {
        Self { spend_by_model: HashMap::new() }
    }

    pub fn add_spend(&mut self, model: &str, cost_usd: f64) {
        if cost_usd > 0.0 {
            *self.spend_by_model.entry(model.to_string()).or_insert(0.0) += cost_usd;
        }
    }

    pub fn total_spend(&self) -> f64 {
        self.spend_by_model.values().sum()
    }

    pub fn spend_for_model(&self, model: &str) -> f64 {
        *self.spend_by_model.get(model).unwrap_or(&0.0)
    }
}
