# Atomic Token Counter & Tokio Synchronization Specification

## 1. Concurrency Model
High-throughput coding agents frequently execute parallel sub-requests (e.g. multi-file code review, test generations). BurnRate tracks token spend across multiple threads without locking.

## 2. Lock-Free Atomic Primitives
Token counters use standard library 64-bit atomic integers (`AtomicU64`):
```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct SessionMetrics {
    pub prompt_tokens: AtomicU64,
    pub completion_tokens: AtomicU64,
    pub cost_micros: AtomicU64, // Micro-dollars ($0.000001) for integer precision
}
```

## 3. Memory Ordering Guarantees
- Counter updates use `Ordering::Relaxed` for independent additions.
- Budget threshold checks use `Ordering::AcqRel` compare-and-swap (CAS) to prevent budget overruns during concurrent request completions.
