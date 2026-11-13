# Upstream Rate Limiting (HTTP 429) & Full Jitter Retry Specification

## 1. Upstream 429 Handling
When frontier LLM APIs return `429 Too Many Requests`, BurnRate intercepts the response and parses provider rate-limit headers:
- `retry-after`: Seconds until rate-limit bucket replenishment.
- `x-ratelimit-reset-requests`: Timestamp of request quota renewal.

## 2. Exponential Backoff with Decorrelated Jitter
$$\text{Sleep} = \min(T_{\max}, \text{Uniform}(0, T_{\text{base}} \cdot 2^{\text{attempt}}))$$

Where $T_{\text{base}} = 500\,\text{ms}$ and $T_{\max} = 10\,\text{s}$.

## 3. Circuit Breaker Escalation
If 4 consecutive retry attempts fail, BurnRate emits a structured error envelope to the client agent, preventing indefinite execution hangs.
