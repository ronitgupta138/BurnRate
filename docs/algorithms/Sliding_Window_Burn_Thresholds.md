# Sliding-Window Cost-Burn Rate & Tripwire Circuit Breaker Specification

## 1. Burn Rate Mathematical Formulation
BurnRate measures the spending acceleration of autonomous agent coding loops over a 60-second sliding window:

$$\text{BurnRate}_{60s} = \frac{\sum_{i=1}^{K} \text{Cost}(R_i)}{\Delta t} \quad (\$/\text{minute})$$

## 2. Tripwire Invariants
- **Soft Warning Ceiling:** Triggers visual yellow alert when burn rate exceeds $\$0.50/\text{min}$.
- **Hard Circuit Breaker:** Rejects upstream agent requests with HTTP `402 Payment Required` when cumulative session spend crosses the user-configured hard limit (e.g. $\$5.00$).

## 3. Tripwire Reset Protocols
Circuit breakers can be unlocked interactively via the terminal HUD (`press [U] to override`) or via the companion CLI (`burnrate reset --session`).
