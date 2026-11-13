# Upstream API Key Security & Memory Zeroing Specification

## 1. Security Threat Model
Provider API keys (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`) cached in process memory must not leak into core dumps, swap space, or error logs.

## 2. Secure String Storage via Zeroize
- Sensitive credentials implement the `zeroize::Zeroize` trait.
- Key buffers are automatically wiped with zeroes upon `Drop`:
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretApiKey(String);
```

## 3. Log Redaction Rules
HTTP header parsers systematically mask authorization headers:
`Authorization: Bearer sk-ant-...3779` $\to$ `Authorization: Bearer [REDACTED]`
