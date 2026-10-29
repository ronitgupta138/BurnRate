# OpenTelemetry Distributed Trace Spans

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
W3C Trace Context propagation injecting rate-limiting spans into distributed telemetry graphs.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
