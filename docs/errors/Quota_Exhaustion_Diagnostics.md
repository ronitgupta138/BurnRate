# Quota Exhaustion Diagnostics

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Structured diagnostic payloads reporting consumed tokens, tier limits, and refill intervals.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
