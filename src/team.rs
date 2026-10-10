//! Team budget pool governor with multi-agent quota partitioning.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQuota {
    pub agent_id: String,
    pub allocated_budget_usd: f64,
    pub spent_usd: f64,
    pub turns_count: u64,
    pub is_exhausted: bool,
}

pub struct TeamBudgetPool {
    pub total_team_pool_usd: f64,
    quotas: RwLock<HashMap<String, AgentQuota>>,
}

impl TeamBudgetPool {
    pub fn new(total_pool_usd: f64) -> Self {
        Self {
            total_team_pool_usd: total_pool_usd,
            quotas: RwLock::new(HashMap::new()),
        }
    }

    /// Register or partition a quota for an agent.
    pub fn allocate_agent(&self, agent_id: &str, budget_usd: f64) {
        if let Ok(mut map) = self.quotas.write() {
            map.insert(
                agent_id.to_string(),
                AgentQuota {
                    agent_id: agent_id.to_string(),
                    allocated_budget_usd: budget_usd,
                    spent_usd: 0.0,
                    turns_count: 0,
                    is_exhausted: false,
                },
            );
        }
    }

    /// Record spend for a specific agent. Returns true if agent quota exceeded.
    pub fn record_agent_spend(&self, agent_id: &str, cost_usd: f64) -> bool {
        if let Ok(mut map) = self.quotas.write()
            && let Some(quota) = map.get_mut(agent_id) {
                quota.spent_usd += cost_usd;
                quota.turns_count += 1;
                if quota.spent_usd >= quota.allocated_budget_usd {
                    quota.is_exhausted = true;
                }
                return quota.is_exhausted;
        }
        false
    }

    /// Get snapshot of an agent's quota.
    pub fn get_agent_quota(&self, agent_id: &str) -> Option<AgentQuota> {
        let map = self.quotas.read().ok()?;
        map.get(agent_id).cloned()
    }
}
