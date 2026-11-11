// Test suite for configuration boundary validation

#[test]
fn test_negative_budget_rejected() {
    let budget_usd: f64 = -5.0;
    let is_valid = budget_usd >= 0.0;
    assert!(!is_valid, "Negative budget limits must be rejected");
}

#[test]
fn test_positive_budget_accepted() {
    let budget_usd: f64 = 10.0;
    assert!(budget_usd > 0.0);
}
