/// Velocity spike detector identifying sudden cost acceleration in agent loops.
#[derive(Debug, Clone)]
pub struct VelocitySpikeDetector {
    threshold_rate_per_min: f64,
}

impl VelocitySpikeDetector {
    pub fn new(threshold_rate_per_min: f64) -> Self {
        Self { threshold_rate_per_min }
    }

    pub fn is_spike(&self, current_rate: f64) -> bool {
        current_rate > self.threshold_rate_per_min
    }

    pub fn threshold(&self) -> f64 {
        self.threshold_rate_per_min
    }
}
