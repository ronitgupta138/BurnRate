# Rate Limit Exceeded Error Response Schema

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
RFC 7807 compliant Problem Details JSON schema returning retry-after and reset timestamps.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
