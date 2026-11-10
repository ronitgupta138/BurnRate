# Proxy Header Sanitization and Credential Redaction Invariants

## 1. Zero-Leakage Policy
- All outgoing request and response logs pass through a strict regex redactor masking `Authorization`, `x-api-key`, and bearer tokens (`sk-ant-...`, `sk-...`).
- Prevents accidental token exposure in stack traces, system logs, or terminal diagnostic outputs.
