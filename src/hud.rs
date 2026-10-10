//! Terminal HUD status bar rendering.

use crate::budget::BudgetStatus;
use colored::Colorize;

pub struct TerminalHud;

impl TerminalHud {
    /// Render formatted ANSI floating status bar string.
    pub fn render_bar(status: &BudgetStatus) -> String {
        let budget_color = if status.percent_consumed > 90.0 {
            format!("${:.2} / ${:.2}", status.total_spent_usd, status.max_spend_usd).red().bold()
        } else if status.percent_consumed > 65.0 {
            format!("${:.2} / ${:.2}", status.total_spent_usd, status.max_spend_usd).yellow().bold()
        } else {
            format!("${:.2} / ${:.2}", status.total_spent_usd, status.max_spend_usd).green().bold()
        };

        let status_badge = if status.circuit_breaker_tripped {
            "HALTED (BUDGET CAP REACHED)".red().bold()
        } else if status.cache_hit_rate > 70.0 {
            "OPTIMAL (Prompt Cache Warm)".green().bold()
        } else {
            "ACTIVE".cyan().bold()
        };

        format!(
            "┌─────────────────────────────────────────────────────────────────────────────┐\n\
             │ burnrate │ Session: {:<21} │ Burn: ${:<5.2}/min │ Cache: {:<4.1}%   │\n\
             │ Tokens: {:<6} total │ Turns: {:<3} │ Status: {:<35} │\n\
             └─────────────────────────────────────────────────────────────────────────────┘",
            budget_color,
            status.burn_rate_usd_per_min,
            status.cache_hit_rate,
            format!("{:.1}k", status.total_tokens as f64 / 1000.0),
            status.total_turns,
            status_badge
        )
    }

    /// Print HUD directly to stderr so it does not pollute piped agent stdout.
    pub fn display(status: &BudgetStatus) {
        eprintln!("{}", Self::render_bar(status));
    }
}
