use burnrate::budget::BudgetGovernor;
use burnrate::metrics::MetricsExporter;

#[test]
fn test_prometheus_metrics_export() {
    let gov = BudgetGovernor::new(5.00);
    gov.record_turn(1.25, 40000, 3000, 35000);
    let status = gov.status();

    let text = MetricsExporter::render_prometheus(&status);
    assert!(text.contains("burnrate_total_spent_usd 1.250000"));
    assert!(text.contains("burnrate_max_spend_usd 5.00"));
    assert!(text.contains("burnrate_turns_total 1"));
    assert!(text.contains("burnrate_circuit_breaker_tripped 0"));
}
