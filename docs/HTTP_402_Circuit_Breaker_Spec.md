# HTTP 402 Payment Required Circuit Breaker Specification

## 1. Overview
BurnRate enforces proactive budget defense for autonomous coding agents (Claude Code, Hermes, Cursor) by intercepting outgoing LLM inference requests and asserting that token expenditures remain within user-defined fiscal boundaries.

## 2. Multi-Tiered Threshold Cascade
- **Tier 1 (50% Budget Utilization - Informational):**
  - Response headers include `X-BurnRate-Warning: 50_PERCENT_EXCEEDED` and current dollar balance. No throttling applied.
- **Tier 2 (80% Budget Utilization - Soft Throttle):**
  - Injects a synthetic 1,500ms jitter delay on outbound requests and logs an alert to stderr/syslog to discourage infinite tool loops.
- **Tier 3 (100% Budget Utilization - Hard Circuit Trip):**
  - Outbound requests are short-circuited immediately with HTTP status code `402 Payment Required`.
  - Upstream LLM provider is never contacted, guaranteeing zero additional billing.

## 3. Circuit Breaker State Transitions
- States: `CLOSED` (normal operation), `HALF_OPEN` (provisional probe request on quota reset), and `OPEN` (traffic hard-blocked).
- Window Resets: Sliding 1-hour, 24-hour, and monthly billing cycles.
