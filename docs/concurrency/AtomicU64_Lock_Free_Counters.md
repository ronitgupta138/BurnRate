# AtomicU64 Lock Free State Representation

## Overview
Technical specification for **BurnRate** systems engine.

## Specification Details
Packing token count and microsecond timestamps into a single 64-bit atomic integer using CAS loops.

## Invariants & Design Guarantees
- Zero-cost abstractions where possible.
- Safe Rust memory semantics without undefined behavior (`unsafe` strictly audited).
- Deterministic behavior under concurrent thread interleavings.
