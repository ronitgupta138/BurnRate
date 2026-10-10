//! Dynamic sliding-window velocity brake to throttle runaway token loops.

use std::collections::VecDeque;
use std::sync::RwLock;
use std::time::Instant;

/// Sliding-window token velocity brake.
pub struct VelocityBrake {
    pub max_tokens_per_minute: u64,
    window_duration_secs: u64,
    history: RwLock<VecDeque<(Instant, u64)>>,
}

impl VelocityBrake {
    pub fn new(max_tokens_per_minute: u64) -> Self {
        Self {
            max_tokens_per_minute,
            window_duration_secs: 60,
            history: RwLock::new(VecDeque::new()),
        }
    }

    /// Record newly observed tokens. Returns true if velocity exceeded.
    pub fn record_tokens(&self, tokens: u64) -> bool {
        let now = Instant::now();
        if let Ok(mut lock) = self.history.write() {
            lock.push_back((now, tokens));
            self.prune(&mut lock, now);

            let current_window_tokens: u64 = lock.iter().map(|(_, t)| t).sum();
            return current_window_tokens > self.max_tokens_per_minute;
        }
        false
    }

    /// Calculate current rolling token velocity per minute.
    pub fn current_velocity_per_min(&self) -> u64 {
        let now = Instant::now();
        if let Ok(mut lock) = self.history.write() {
            self.prune(&mut lock, now);
            return lock.iter().map(|(_, t)| t).sum();
        }
        0
    }

    fn prune(&self, deque: &mut VecDeque<(Instant, u64)>, now: Instant) {
        while let Some(&(timestamp, _)) = deque.front() {
            if now.duration_since(timestamp).as_secs() > self.window_duration_secs {
                deque.pop_front();
            } else {
                break;
            }
        }
    }
}
