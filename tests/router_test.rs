use burnrate::router::ModelCostRouter;

#[test]
fn test_model_cost_router_downgrade_on_low_budget() {
    let router = ModelCostRouter::new(0.50); // $0.50 warning threshold

    // 1. High budget, complex task -> Sonnet
    let decision_complex = router.route_task("refactor whole database architecture layer", 2.00);
    assert_eq!(decision_complex.selected_model, "claude-3-7-sonnet-20250219");
    assert!(!decision_complex.downgraded);

    // 2. Low budget (< 0.50), same task -> Downgrade to Haiku
    let decision_downgrade = router.route_task("refactor whole database architecture layer", 0.20);
    assert_eq!(decision_downgrade.selected_model, "claude-3-5-haiku-20241022");
    assert!(decision_downgrade.downgraded);
}
