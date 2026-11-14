use burnrate::budget::calculate_circuit_backoff_ms;

#[test]
fn test_calculate_circuit_backoff_exponential() {
    assert_eq!(calculate_circuit_backoff_ms(0, 100, 5000), 100);
    assert_eq!(calculate_circuit_backoff_ms(1, 100, 5000), 200);
    assert_eq!(calculate_circuit_backoff_ms(2, 100, 5000), 400);
    assert_eq!(calculate_circuit_backoff_ms(3, 100, 5000), 800);
}

#[test]
fn test_calculate_circuit_backoff_cap() {
    assert_eq!(calculate_circuit_backoff_ms(10, 100, 1000), 1000);
}
