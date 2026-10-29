# Sliding Window Log Rate Limiting

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Sorted set timestamp sliding window tracking exact request arrivals over sliding interval T.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
