/// Linear run-rate cost projection to month end.
pub struct LinearCostForecaster;

impl LinearCostForecaster {
    pub fn project_monthly_spend(spend_to_date: f64, current_day: u32, total_days_in_month: u32) -> f64 {
        if current_day == 0 || total_days_in_month == 0 {
            return spend_to_date;
        }
        let daily_run_rate = spend_to_date / (current_day as f64);
        daily_run_rate * (total_days_in_month as f64)
    }
}
