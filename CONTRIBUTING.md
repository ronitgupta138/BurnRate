# Contributing to `burnrate`

Thank you for your interest in contributing to `burnrate`! We welcome contributions to pricing tables, budget governors, prompt-cache linters, terminal HUDs, and client SDKs.

---

## 🛠️ Development Setup

### Prerequisites
- **Rust Toolchain:** 1.80+ (edition 2024 / 2021)
- **Cargo:** Included with Rust

### Build & Test
```bash
# Clone the repository
git clone https://github.com/ronitgupta138/burnrate.git
cd burnrate

# Run the test suite
cargo test

# Check code formatting and lints
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

---

## 📐 Architecture Guidelines

1. **Zero-Latency Proxy Invariant:** The reverse proxy layer (`src/proxy.rs`, `src/streaming.rs`) must stream request/response chunks without unneeded buffering or introducing perceptible latency.
2. **Hard Circuit Breaker Guarantee:** Once `max_spend` is reached, downstream execution must halt immediately with HTTP 402 to prevent surprise bills.
3. **Accurate Cache Discounting:** Pricing models must reflect real 90% prompt-cache read discounts provided by frontier model providers.

---

## 🚀 Pull Request Checklist

- [ ] All unit and integration tests pass (`cargo test`).
- [ ] No compiler or clippy warnings (`cargo clippy --all-targets -- -D warnings`).
- [ ] Added tests for any new pricing tables or governors.
- [ ] Clean commit messages following Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`).
- [ ] Zero local environment paths or hardcoded API keys.
