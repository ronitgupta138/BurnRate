# Client Identity Resolution Protocol

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Resolving client identity via API keys, process credentials, and session tokens.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
