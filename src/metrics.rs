//! Prometheus metrics exporter for enterprise FinOps monitoring.

use crate::budget::BudgetStatus;

pub struct MetricsExporter;

impl MetricsExporter {
    /// Render metrics in Prometheus text exposition format.
    pub fn render_prometheus(status: &BudgetStatus) -> String {
        format!(
            "# HELP burnrate_total_spent_usd Total USD spend accumulated in session\n\
             # TYPE burnrate_total_spent_usd counter\n\
             burnrate_total_spent_usd {:.6}\n\
             \n\
             # HELP burnrate_max_spend_usd Configured hard budget limit in USD\n\
             # TYPE burnrate_max_spend_usd gauge\n\
             burnrate_max_spend_usd {:.2}\n\
             \n\
             # HELP burnrate_remaining_usd Remaining available budget in USD\n\
             # TYPE burnrate_remaining_usd gauge\n\
             burnrate_remaining_usd {:.6}\n\
             \n\
             # HELP burnrate_burn_rate_usd_per_min Real-time spend velocity in USD per minute\n\
             # TYPE burnrate_burn_rate_usd_per_min gauge\n\
             burnrate_burn_rate_usd_per_min {:.6}\n\
             \n\
             # HELP burnrate_turns_total Total executed agent turns\n\
             # TYPE burnrate_turns_total counter\n\
             burnrate_turns_total {}\n\
             \n\
             # HELP burnrate_tokens_total Total tokens processed (input + output + cache)\n\
             # TYPE burnrate_tokens_total counter\n\
             burnrate_tokens_total {}\n\
             \n\
             # HELP burnrate_cache_hit_rate_percent Prompt cache hit ratio percentage\n\
             # TYPE burnrate_cache_hit_rate_percent gauge\n\
             burnrate_cache_hit_rate_percent {:.2}\n\
             \n\
             # HELP burnrate_circuit_breaker_tripped 1 if budget cap exceeded, 0 otherwise\n\
             # TYPE burnrate_circuit_breaker_tripped gauge\n\
             burnrate_circuit_breaker_tripped {}\n",
            status.total_spent_usd,
            status.max_spend_usd,
            status.remaining_usd,
            status.burn_rate_usd_per_min,
            status.total_turns,
            status.total_tokens,
            status.cache_hit_rate,
            if status.circuit_breaker_tripped { 1 } else { 0 }
        )
    }
}
