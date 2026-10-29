# Fixed Window Counter Tradeoffs

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Analysis of edge-boundary 2x burst vulnerability in fixed time windows and boundary mitigations.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
