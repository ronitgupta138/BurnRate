# DuckDB Columnar Analytical Spend Export Specification

## 1. Columnar Parquet Export
- Flushes SQLite spend logs into compressed Parquet files for fast OLAP aggregation in DuckDB.
- Delivers 100x faster analytical scans over historical multi-month usage trends.
