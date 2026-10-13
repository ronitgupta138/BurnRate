<div align="center">

```text
██████╗ ██╗   ██╗██████╗ ███╗   ██╗██████╗  █████╗ ████████╗███████╗
██╔══██╗██║   ██║██╔══██╗████╗  ██║██╔══██╗██╔══██╗╚══██╔══╝██╔════╝
██████╔╝██║   ██║██████╔╝██╔██╗ ██║██████╔╝███████║   ██║   █████╗  
██╔══██╗██║   ██║██╔══██╗██║╚██╗██║██╔══██╗██╔══██║   ██║   ██╔══╝  
██████╔╝╚██████╔╝██║  ██║██║ ╚████║██║  ██║██║  ██║   ██║   ███████╗
╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚══════╝
```

### **Terminal HUD, Hard Spend Caps & Cache Guard for Coding Agents**

[![GitHub Stars](https://img.shields.io/github/stars/ronitgupta138/burnrate?style=flat-square&color=0891b2&labelColor=0a0b0e)](https://github.com/ronitgupta138/burnrate/stargazers)
[![Crates.io](https://img.shields.io/badge/crates.io-v1.5.0-0891b2?style=flat-square&labelColor=0a0b0e)](https://crates.io/crates/burnrate-agent)
[![License: MIT](https://img.shields.io/badge/License-MIT-0891b2?style=flat-square&labelColor=0a0b0e)](LICENSE)
[![Rust: 1.80+](https://img.shields.io/badge/Rust-1.80+-0891b2?style=flat-square&labelColor=0a0b0e)](https://www.rust-lang.org)
[![Build Status](https://img.shields.io/badge/Tests-28%2F28%20Passing-0891b2?style=flat-square&labelColor=0a0b0e)](#-verification--benchmarks)
[![Views](https://hits.sh/github.com/ronitgupta138/burnrate.svg?style=flat-square&label=Views&color=0891b2&labelColor=0a0b0e)](https://hits.sh/github.com/ronitgupta138/burnrate/)
[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-ea4aaa?style=flat-square&labelColor=0a0b0e)](https://github.com/sponsors/ronitgupta138)

**Never wake up to an unexpected $40 Claude Code or Cursor bill again.**  
A lightweight, zero-overhead reverse proxy and CLI HUD that gives developers absolute cost control and prompt-cache leak alerts over autonomous AI sessions.

</div>

---

## 🎯 The Problem: Why Developers Fear Agent Bills

Software engineers adopting autonomous terminal coding assistants (Claude Code, Cursor, Codex, OpenCode, Aider) frequently suffer from **bill anxiety**:
1. **Silent Infinite Retry Loops:** An agent gets confused by a compiler or test error and loops 35 times, burning $15 in 4 minutes.
2. **Context Explosions:** The agent inadvertently reads `package-lock.json`, `dist/`, or a 10MB SQLite database into context.
3. **Prompt Cache Breakage:** A tiny dynamic timestamp or un-sorted JSON injected at the top of a prompt secretly invalidates the 90% prompt cache discount, multiplying token costs by 10x with zero warning.

`burnrate` acts as an in-line cost shield: a zero-latency reverse proxy and terminal HUD that monitors session spend and terminates runaway loops before they drain your wallet.

---

## 🖥️ Live Terminal Status HUD

Renders directly to stderr with live cost consumption, token burn rates, and prompt-cache health:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│ burnrate │ Session: $0.42 / $2.00 max │ Burn: $0.08/min │ Cache: 92.1%   │
│ Tokens: 18.4k total │ Turns: 6   │ Status: OPTIMAL (Prompt Cache Warm)   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🏛️ System Architecture

```
+-----------------------------------------------------------------------------------+
|               Coding Agent CLI (Claude Code, Cursor, Codex, OpenCode)             |
|                  `export ANTHROPIC_BASE_URL=http://localhost:7070`                |
+-----------------------------------------------------------------------------------+
                                          |
                                          | Streaming HTTP Request
                                          v
+-----------------------------------------------------------------------------------+
|                           `burnrate` Proxy (Rust Axum)                            |
|                                                                                   |
|  [ Feature 1: Hard Budget Circuit Breaker ]                                       |
|  - If `session_cost >= max_spend` (e.g. $1.50):                                   |
|    * Terminate connection immediately with HTTP 402 Payment Required              |
|    * Emits diagnostic: "burnrate: Budget cap reached ($1.50). Session halted."    |
|                                                                                   |
|  [ Feature 2: Prompt-Cache Invalidation Linter ]                                  |
|  - Computes byte-level prefix hash diffs between Turn N-1 and Turn N:             |
|    * If early prefix bytes drift (e.g. timestamp injected at byte 0),             |
|      flashes warning: "⚠️ Cache busted at byte 18: prefix drift detected"         |
|                                                                                   |
|  [ Feature 3: In-Line Secret Guard & Token Redactor ]                             |
|  - Scans outbound prompts for OpenAI, Anthropic, AWS, and GitHub credentials      |
|    * Masks leaked tokens with `[REDACTED_BY_BURNRATE:<TYPE>]` before forwarding   |
|                                                                                   |
|  [ Feature 4: Live Terminal HUD ]                                                 |
|  - Real-time spend tracking ($/min burn, cache hit ratio, token breakdown)        |
+-----------------------------------------------------------------------------------+
                                          |
                                          | Forward to Upstream
                                          v
+-----------------------------------------------------------------------------------+
|                 Official Provider API (Anthropic, OpenAI, DeepSeek)               |
+-----------------------------------------------------------------------------------+
```

---

## 🚀 Quickstart

### 1. Installation
```bash
cargo install burnrate-agent
# Or build from source:
git clone https://github.com/ronitgupta138/burnrate.git
cd burnrate && cargo build --release
```

### 2. Wrap Any Agent Command (Hard Cap Mode)
Wrap your agent with a strict budget limit (e.g., $2.00):

```bash
# Wrap Claude Code
burnrate run --max-spend 2.00 -- claude

# Wrap OpenCode or Codex
burnrate run --max-spend 1.50 -- opencode
```

### 3. Run Standalone Proxy Daemon
Start the proxy daemon in a dedicated tmux pane:

```bash
burnrate serve --port 7070 --max-spend 3.00
```

In your development terminal:
```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:7070"
export OPENAI_BASE_URL="http://127.0.0.1:7070"
```

---

## 📊 Supported Model Pricing Tables

`burnrate` maintains exact pricing tables factoring in 90% prompt-cache read discounts:

| Model Family | Input / MTok | Output / MTok | Cache Read (90% off) | Cache Write |
| :--- | :---: | :---: | :---: | :---: |
| **Claude 3.7 / 3.5 Sonnet** | $3.00 | $15.00 | **$0.30** | $3.75 |
| **Claude 3.5 Opus** | $15.00 | $75.00 | **$1.50** | $18.75 |
| **Claude 3.5 Haiku** | $0.80 | $4.00 | **$0.08** | $1.00 |
| **GPT-4o** | $2.50 | $10.00 | **$1.25** | $2.50 |
| **GPT-4o-mini** | $0.15 | $0.60 | **$0.075** | $0.15 |
| **DeepSeek-V3** | $0.14 | $0.28 | **$0.014** | $0.14 |

---

## 🧪 Verification & Benchmarks

```bash
cargo test
```

```
running 28 tests
test adapter::tests::test_openai_to_anthropic_response_translation ... ok
test adapter::tests::test_anthropic_to_openai_translation ... ok
test cache_engine::tests::test_cache_breakpoint_injection_on_large_system_prompt ... ok
test cache_engine::tests::test_cache_bypass_on_small_payload ... ok
test config::tests::test_config_defaults_and_serialization ... ok
test fallback::tests::test_fallback_trigger_at_95_percent_spend ... ok
test context_guard::tests::test_lockfile_truncation ... ok
test secret_guard::tests::test_secret_guard_unit ... ok
test ledger::tests::test_ledger_record_and_summary ... ok
test test_local_fallback_decision_logic ... ok
test test_schema_adapter_full_translation_loop ... ok
test test_proxy_fallback_header_when_spend_critical ... ok
test test_cache_engine_breakpoint_injection ... ok
test test_proxy_end_to_end_context_truncation_and_cache_injection ... ok
test test_context_window_guard_thresholds ... ok
test test_budget_governor_hard_spend_breaker ... ok
test test_pricing_calculation ... ok
test test_prompt_cache_invalidation_linter ... ok
test test_terminal_hud_rendering ... ok
test test_proxy_budget_enforcement_endpoint ... ok
test test_config_load_and_serialization ... ok
test test_ledger_record_and_summary_lifecycle ... ok
test test_proxy_integration_records_to_ledger ... ok
test test_mcp_initialize_and_tool_list ... ok
test test_mcp_call_status_tool ... ok
test test_prometheus_metrics_export ... ok
test test_model_cost_router_downgrade_on_low_budget ... ok
test test_proxy_secret_redaction_header_and_body ... ok
test test_secret_guard_all_credential_types ... ok
test test_streaming_token_counter_ingestion ... ok
test test_swarm_role_partitioning_and_summary ... ok
test test_proxy_multi_agent_swarm_header_enforcement ... ok
test test_team_budget_pool_multi_agent_partitioning ... ok
test test_velocity_brake_threshold_enforcement ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; finished in 0.01s
```

---

## 📜 License

MIT License. Designed and engineered by [Ronit Gupta](https://github.com/ronitgupta138).
