# Prometheus Token Cost Exporter Metrics Specification

## 1. Metric Hierarchy
BurnRate exposes an embedded Prometheus scrape endpoint at `/metrics` (default port: `9182`):

- `burnrate_prompt_tokens_total` (Counter):
  - Labels: `provider`, `model`, `agent_session`, `status`
- `burnrate_completion_tokens_total` (Counter):
  - Labels: `provider`, `model`, `agent_session`, `status`
- `burnrate_cached_prompt_tokens_total` (Counter):
  - Labels: `provider`, `model`
- `burnrate_estimated_cost_usd_total` (Counter):
  - Labels: `provider`, `model`
- `burnrate_circuit_breaker_tripped_total` (Counter):
  - Labels: `threshold_tier`

## 2. Grafana Dashboard Integration
PromQL alert rule template:
```promql
sum(rate(burnrate_estimated_cost_usd_total[5m])) by (model) > 0.50
```
Triggers pager notification if token burn rate exceeds $0.50 USD per minute.
