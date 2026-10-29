# Model Context Protocol Integration

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Standardized JSON-RPC 2.0 tool call interceptor and quota governor for Claude Code and Hermes Agent.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
