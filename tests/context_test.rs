use burnrate::context_guard::ContextWindowGuard;

#[test]
fn test_context_window_guard_thresholds() {
    let guard = ContextWindowGuard::new(100, 500);

    // 1. Small benign payload (40 bytes ~ 10 tokens) -> None
    let small = vec![b'a'; 40];
    assert!(guard.inspect_payload(&small).is_none());

    // 2. Medium warning payload (800 bytes ~ 200 tokens) -> WARNING
    let medium = vec![b'b'; 800];
    let alert_med = guard.inspect_payload(&medium).unwrap();
    assert_eq!(alert_med.severity, "WARNING");

    // 3. Large critical explosion (2400 bytes ~ 600 tokens) -> CRITICAL
    let large = vec![b'c'; 2400];
    let alert_crit = guard.inspect_payload(&large).unwrap();
    assert_eq!(alert_crit.severity, "CRITICAL");
}
