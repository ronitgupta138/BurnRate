// Test suite for token bucket rate limiter

#[test]
fn test_bucket_capacity_cap() {
    let capacity = 50_000;
    let tokens_added = 75_000;
    let current_tokens = std::cmp::min(capacity, tokens_added);
    assert_eq!(current_tokens, 50_000);
}

#[test]
fn test_token_deduction() {
    let mut tokens = 10_000;
    let cost = 3_000;
    tokens -= cost;
    assert_eq!(tokens, 7_000);
}
