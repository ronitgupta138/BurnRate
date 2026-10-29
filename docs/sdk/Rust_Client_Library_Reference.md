# Rust Client Library SDK Reference

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Ergonomic typed Rust client providing async middleware wrappers for hyper, axum, and reqwest.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
