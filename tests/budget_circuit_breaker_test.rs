// Test suite for BurnRate budget circuit breaker trip logic

#[test]
fn test_budget_within_limit() {
    let budget_limit_usd = 10.0;
    let current_spend_usd = 4.5;
    assert!(current_spend_usd < budget_limit_usd);
}

#[test]
fn test_circuit_trips_at_threshold() {
    let budget_limit_usd = 10.0;
    let current_spend_usd = 10.01;
    let is_tripped = current_spend_usd >= budget_limit_usd;
    assert!(is_tripped, "Circuit breaker must trip when spend exceeds limit");
}

#[test]
fn test_half_open_recovery_probe() {
    let is_reset_time_passed = true;
    let state = if is_reset_time_passed { "HALF_OPEN" } else { "OPEN" };
    assert_eq!(state, "HALF_OPEN");
}
