use burnrate::velocity::VelocityBrake;

#[test]
fn test_velocity_brake_threshold_enforcement() {
    let brake = VelocityBrake::new(1000); // 1,000 tokens/min limit

    // 1. Safe burst of 400 tokens -> false
    assert!(!brake.record_tokens(400));
    assert_eq!(brake.current_velocity_per_min(), 400);

    // 2. Safe burst of 400 tokens (total 800) -> false
    assert!(!brake.record_tokens(400));
    assert_eq!(brake.current_velocity_per_min(), 800);

    // 3. Runaway burst of 500 tokens (total 1300 > 1000) -> true (exceeded!)
    assert!(brake.record_tokens(500));
    assert_eq!(brake.current_velocity_per_min(), 1300);
}
