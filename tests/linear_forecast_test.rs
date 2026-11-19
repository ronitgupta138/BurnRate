use burnrate::linear_forecast::LinearCostForecaster;

#[test]
fn test_monthly_projection() {
    // Spent $10.00 by day 10 of a 30-day month -> $1.00/day * 30 days = $30.00
    let projected = LinearCostForecaster::project_monthly_spend(10.0, 10, 30);
    assert!((projected - 30.0).abs() < 1e-5);
}
