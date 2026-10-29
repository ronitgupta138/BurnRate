# Prometheus Metrics Exporter Spec

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Standard OpenMetrics exposition format exposing counters for allowed, throttled, and rejected calls.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
