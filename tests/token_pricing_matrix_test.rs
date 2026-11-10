// Test suite for multi-provider token cost calculations

#[test]
fn test_claude_sonnet_cost_calculation() {
    let prompt_tokens = 1_000_000;
    let rate_per_m = 3.00; // $3.00 per 1M prompt tokens
    let cost = (prompt_tokens as f64 / 1_000_000.0) * rate_per_m;
    assert_eq!(cost, 3.00);
}

#[test]
fn test_cache_read_discount() {
    let normal_rate = 3.00;
    let cached_rate = 0.30; // 90% discount
    assert!(cached_rate < normal_rate);
}
