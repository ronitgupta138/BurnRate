//! Team budget pool governor with multi-agent quota partitioning and swarm fleet governance.

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Standard role classification in an autonomous agent swarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Architect,
    Worker,
    Tester,
    Reviewer,
}

impl AgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Architect => "Architect",
            Self::Worker => "Worker",
            Self::Tester => "Tester",
            Self::Reviewer => "Reviewer",
        }
    }

    /// Default quota percentage share of the team budget pool.
    pub fn default_pool_share(&self) -> f64 {
        match self {
            Self::Architect => 0.50, // 50% for high-context planning
            Self::Worker => 0.30,    // 30% for implementation
            Self::Tester => 0.15,    // 15% for verification & test runs
            Self::Reviewer => 0.05,  // 5% for lint & final sign-off
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "architect" | "lead" | "planner" => Self::Architect,
            "tester" | "qa" => Self::Tester,
            "reviewer" | "audit" => Self::Reviewer,
            _ => Self::Worker,
        }
    }
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentQuota {
    pub agent_id: String,
    pub role: AgentRole,
    pub allocated_budget_usd: f64,
    pub spent_usd: f64,
    pub turns_count: u64,
    pub is_exhausted: bool,
}

/// Cumulative summary across the entire swarm budget pool.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwarmSummary {
    pub total_pool_usd: f64,
    pub total_spent_usd: f64,
    pub remaining_pool_usd: f64,
    pub percent_consumed: f64,
    pub registered_agents_count: usize,
    pub exhausted_agents_count: usize,
    pub pool_exhausted: bool,
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
        self.allocate_agent_with_role(agent_id, AgentRole::Worker, budget_usd);
    }

    /// Register an agent with an explicit swarm role and budget limit.
    pub fn allocate_agent_with_role(&self, agent_id: &str, role: AgentRole, budget_usd: f64) {
        if let Ok(mut map) = self.quotas.write() {
            map.insert(
                agent_id.to_string(),
                AgentQuota {
                    agent_id: agent_id.to_string(),
                    role,
                    allocated_budget_usd: budget_usd,
                    spent_usd: 0.0,
                    turns_count: 0,
                    is_exhausted: false,
                },
            );
        }
    }

    /// Auto-enroll an unannounced agent using role-based pool defaults.
    pub fn enroll_or_get(&self, agent_id: &str, role: AgentRole) -> AgentQuota {
        {
            #[allow(clippy::collapsible_if)]
            if let Ok(map) = self.quotas.read() {
                if let Some(q) = map.get(agent_id) {
                    return q.clone();
                }
            }
        }

        let allocated = (self.total_team_pool_usd * role.default_pool_share()).max(0.50);
        self.allocate_agent_with_role(agent_id, role, allocated);

        self.get_agent_quota(agent_id)
            .unwrap_or_else(|| AgentQuota {
                agent_id: agent_id.to_string(),
                role,
                allocated_budget_usd: allocated,
                spent_usd: 0.0,
                turns_count: 0,
                is_exhausted: false,
            })
    }

    /// Record spend for a specific agent. Returns true if agent quota exceeded.
    pub fn record_agent_spend(&self, agent_id: &str, cost_usd: f64) -> bool {
        let (agent_exhausted, _) = self.record_spend_with_role(agent_id, AgentRole::Worker, cost_usd);
        agent_exhausted
    }

    /// Record spend for an agent with dynamic swarm role tracking.
    /// Returns `(agent_exhausted, total_pool_exhausted)`.
    pub fn record_spend_with_role(
        &self,
        agent_id: &str,
        role: AgentRole,
        cost_usd: f64,
    ) -> (bool, bool) {
        let mut agent_exhausted = false;
        let mut total_spent = 0.0;

        if let Ok(mut map) = self.quotas.write() {
            let quota = map.entry(agent_id.to_string()).or_insert_with(|| {
                let alloc = (self.total_team_pool_usd * role.default_pool_share()).max(0.50);
                AgentQuota {
                    agent_id: agent_id.to_string(),
                    role,
                    allocated_budget_usd: alloc,
                    spent_usd: 0.0,
                    turns_count: 0,
                    is_exhausted: false,
                }
            });

            quota.spent_usd += cost_usd;
            quota.turns_count += 1;
            if quota.spent_usd >= quota.allocated_budget_usd {
                quota.is_exhausted = true;
            }
            agent_exhausted = quota.is_exhausted;

            for q in map.values() {
                total_spent += q.spent_usd;
            }
        }

        let total_exhausted = total_spent >= self.total_team_pool_usd;
        (agent_exhausted, total_exhausted)
    }

    /// Get snapshot of an agent's quota.
    pub fn get_agent_quota(&self, agent_id: &str) -> Option<AgentQuota> {
        let map = self.quotas.read().ok()?;
        map.get(agent_id).cloned()
    }

    /// Compute cumulative swarm status across all agents.
    pub fn swarm_status(&self) -> SwarmSummary {
        let mut total_spent = 0.0;
        let mut count = 0;
        let mut exhausted = 0;

        if let Ok(map) = self.quotas.read() {
            count = map.len();
            for q in map.values() {
                total_spent += q.spent_usd;
                if q.is_exhausted {
                    exhausted += 1;
                }
            }
        }

        let remaining = (self.total_team_pool_usd - total_spent).max(0.0);
        let pct = if self.total_team_pool_usd > 0.0 {
            (total_spent / self.total_team_pool_usd) * 100.0
        } else {
            0.0
        };

        SwarmSummary {
            total_pool_usd: self.total_team_pool_usd,
            total_spent_usd: total_spent,
            remaining_pool_usd: remaining,
            percent_consumed: pct,
            registered_agents_count: count,
            exhausted_agents_count: exhausted,
            pool_exhausted: total_spent >= self.total_team_pool_usd,
        }
    }

    /// Render human-readable ASCII table of registered swarm agents.
    pub fn render_swarm_table(&self) -> String {
        let quotas = match self.quotas.read() {
            Ok(map) => map.values().cloned().collect::<Vec<_>>(),
            Err(_) => return "Failed to read swarm quotas".to_string(),
        };

        if quotas.is_empty() {
            return "No agents currently registered in swarm budget pool.".dimmed().to_string();
        }

        let mut out = String::new();
        out.push_str(&format!(
            "┌{0:─<18}┬{0:─<14}┬{0:─<14}┬{0:─<14}┬{0:─<8}┬{0:─<12}┐\n",
            ""
        ));
        out.push_str(&format!(
            "│ {: <16} │ {: <12} │ {: <12} │ {: <12} │ {: <6} │ {: <10} │\n",
            "AGENT ID".bold(),
            "ROLE".bold(),
            "SPENT".bold(),
            "ALLOCATED".bold(),
            "TURNS".bold(),
            "STATUS".bold()
        ));
        out.push_str(&format!(
            "├{0:─<18}┼{0:─<14}┼{0:─<14}┼{0:─<14}┼{0:─<8}┼{0:─<12}┤\n",
            ""
        ));

        for q in quotas {
            let spent_str = format!("${:.2}", q.spent_usd);
            let alloc_str = format!("${:.2}", q.allocated_budget_usd);
            let status_str = if q.is_exhausted {
                "EXHAUSTED".red().bold().to_string()
            } else {
                "ACTIVE".green().to_string()
            };

            out.push_str(&format!(
                "│ {: <16} │ {: <12} │ {: <12} │ {: <12} │ {: <6} │ {: <10} │\n",
                q.agent_id.chars().take(16).collect::<String>(),
                q.role.as_str(),
                spent_str,
                alloc_str,
                q.turns_count,
                status_str
            ));
        }

        out.push_str(&format!(
            "└{0:─<18}┴{0:─<14}┴{0:─<14}┴{0:─<14}┴{0:─<8}┴{0:─<12}┘",
            ""
        ));

        out
    }
}
