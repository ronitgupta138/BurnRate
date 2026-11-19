use burnrate::daily_spend::DailySpendAggregator;

#[test]
fn test_spend_aggregation() {
    let mut agg = DailySpendAggregator::new();
    agg.add_spend("claude-sonnet", 0.05);
    agg.add_spend("claude-sonnet", 0.10);
    agg.add_spend("gpt-4o", 0.20);

    assert!((agg.total_spend() - 0.35).abs() < 1e-5);
    assert!((agg.spend_for_model("claude-sonnet") - 0.15).abs() < 1e-5);
    assert_eq!(agg.spend_for_model("unknown"), 0.0);
}
