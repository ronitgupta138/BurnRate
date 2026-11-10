# Budget Inspection MCP Tool Specification

## 1. Tool Declaration
- `burnrate_get_status`:
  - Returns current window utilization, remaining balance, and circuit breaker trip state.
- `burnrate_set_limit`:
  - Dynamically updates active hourly/daily ceiling.
