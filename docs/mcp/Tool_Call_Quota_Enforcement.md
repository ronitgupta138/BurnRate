# Tool Call Quota Enforcement Rules

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Per-tool rate limiting allowing granular limits on high-cost terminal and web browsing executions.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
