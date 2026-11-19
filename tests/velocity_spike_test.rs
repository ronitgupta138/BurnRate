use burnrate::spike_detector::VelocitySpikeDetector;

#[test]
fn test_spike_detection() {
    let detector = VelocitySpikeDetector::new(0.50); // $0.50/min
    assert!(!detector.is_spike(0.40));
    assert!(!detector.is_spike(0.50));
    assert!(detector.is_spike(0.51));
    assert!(detector.is_spike(1.20));
}
