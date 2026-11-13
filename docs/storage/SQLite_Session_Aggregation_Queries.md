# SQLite WAL Session Storage & Financial Aggregation Specification

## 1. Local Audit Ledger
BurnRate records every completed LLM request and token breakdown into a local SQLite database located at `~/.burnrate/history.db`.

## 2. Write-Ahead Logging (WAL) Configuration
- `PRAGMA journal_mode = WAL;` (Enables concurrent non-blocking reads during active writes).
- `PRAGMA synchronous = NORMAL;` (Ensures zero I/O stutter on terminal typing threads).
- `PRAGMA busy_timeout = 5000;`

## 3. Analytical Query Specification
Daily and per-project cost rollups execute via optimized indexes:
```sql
CREATE INDEX idx_requests_session ON requests(session_id, timestamp);
SELECT project_id, SUM(prompt_tokens), SUM(completion_tokens), SUM(cost_usd)
FROM requests WHERE timestamp >= :day_start GROUP BY project_id;
```
