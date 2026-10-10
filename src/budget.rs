//! Hard spend budget circuit breaker.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Active budget governor state.
pub struct BudgetGovernor {
    pub max_spend_cents: u64,
    pub spent_micros: AtomicU64,
    pub total_input_tokens: AtomicU64,
    pub total_output_tokens: AtomicU64,
    pub total_cache_read_tokens: AtomicU64,
    pub total_turns: AtomicU64,
    start_time: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub total_spent_usd: f64,
    pub max_spend_usd: f64,
    pub remaining_usd: f64,
    pub percent_consumed: f64,
    pub burn_rate_usd_per_min: f64,
    pub total_turns: u64,
    pub total_tokens: u64,
    pub cache_hit_rate: f64,
    pub circuit_breaker_tripped: bool,
}

impl BudgetGovernor {
    pub fn new(max_spend_usd: f64) -> Self {
        let max_spend_cents = (max_spend_usd * 100.0).round() as u64;
        Self {
            max_spend_cents,
            spent_micros: AtomicU64::new(0),
            total_input_tokens: AtomicU64::new(0),
            total_output_tokens: AtomicU64::new(0),
            total_cache_read_tokens: AtomicU64::new(0),
            total_turns: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    /// Record turn cost and tokens. Returns true if budget breached.
    pub fn record_turn(
        &self,
        cost_usd: f64,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
    ) -> bool {
        let micros = (cost_usd * 1_000_000.0).round() as u64;
        self.spent_micros.fetch_add(micros, Ordering::Relaxed);
        self.total_input_tokens.fetch_add(input_tokens, Ordering::Relaxed);
        self.total_output_tokens.fetch_add(output_tokens, Ordering::Relaxed);
        self.total_cache_read_tokens.fetch_add(cache_read_tokens, Ordering::Relaxed);
        self.total_turns.fetch_add(1, Ordering::Relaxed);

        self.is_tripped()
    }

    /// Check if hard spend limit has been exceeded.
    pub fn is_tripped(&self) -> bool {
        let current_micros = self.spent_micros.load(Ordering::Relaxed);
        let max_micros = self.max_spend_cents * 10_000;
        current_micros >= max_micros
    }

    /// Produce a point-in-time snapshot of the session budget.
    pub fn status(&self) -> BudgetStatus {
        let spent_micros = self.spent_micros.load(Ordering::Relaxed);
        let total_spent_usd = spent_micros as f64 / 1_000_000.0;
        let max_spend_usd = self.max_spend_cents as f64 / 100.0;
        let remaining_usd = (max_spend_usd - total_spent_usd).max(0.0);
        let percent_consumed = if max_spend_usd > 0.0 {
            (total_spent_usd / max_spend_usd) * 100.0
        } else {
            0.0
        };

        let elapsed_minutes = (self.start_time.elapsed().as_secs_f64() / 60.0).max(0.01);
        let burn_rate_usd_per_min = total_spent_usd / elapsed_minutes;

        let in_tok = self.total_input_tokens.load(Ordering::Relaxed);
        let out_tok = self.total_output_tokens.load(Ordering::Relaxed);
        let cache_tok = self.total_cache_read_tokens.load(Ordering::Relaxed);
        let total_tokens = in_tok + out_tok + cache_tok;

        let cache_hit_rate = if total_tokens > 0 {
            (cache_tok as f64 / total_tokens as f64) * 100.0
        } else {
            0.0
        };

        BudgetStatus {
            total_spent_usd,
            max_spend_usd,
            remaining_usd,
            percent_consumed,
            burn_rate_usd_per_min,
            total_turns: self.total_turns.load(Ordering::Relaxed),
            total_tokens,
            cache_hit_rate,
            circuit_breaker_tripped: self.is_tripped(),
        }
    }
}
