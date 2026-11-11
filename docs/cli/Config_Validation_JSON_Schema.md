# JSON Schema Specification for BurnRate Configuration

## 1. Schema Validation
Validates config against strict type rules:
- hourly_limit_usd (number >= 0.0).
- trip_action (enum: HALT, THROTTLE, WARN).
