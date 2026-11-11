# OpenTelemetry Distributed Tracing Export Specification

## 1. Tracing Pipeline
- Injects traceparent headers into outgoing LLM provider calls.
- Emits span attributes: llm.provider, llm.model, llm.usage.total_tokens, llm.cost_usd.
