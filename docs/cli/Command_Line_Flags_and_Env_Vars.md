# Command Line Flags and Environment Variables

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Configuration reference for CLI daemon flags (`--port`, `--capacity`, `--refill-rate`, `--redis-url`).

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
