# Strict TLS Certificate Validation & Upstream Proxy Invariants

## 1. Security Invariants
- Upstream connections to AI provider APIs strictly require TLS 1.3 with ALPN (HTTP/2).
- Rejects insecure cipher suites and self-signed certificates in production mode.
