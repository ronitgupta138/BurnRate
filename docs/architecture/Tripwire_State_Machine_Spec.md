# Sliding-Window Cost Tripwire State Machine Specification

## 1. State Machine Definitions
- `NORMAL`: Spend rate within acceptable developer budget.
- `WARNING`: Spend rate exceeds soft alert ceiling ($0.50/min).
- `CIRCUIT_BROKEN`: Cumulative budget exceeded; proxy returns HTTP `402 Payment Required`.

## 2. Transition Rules
Transitions to `CIRCUIT_BROKEN` are instantaneous upon token counting receipt.
