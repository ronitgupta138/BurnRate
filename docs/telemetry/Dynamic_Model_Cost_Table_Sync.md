# Dynamic Model Token Cost Table Synchronization Specification

## 1. Multi-Provider Pricing Normalization
BurnRate evaluates token spend across multiple frontier models (Claude 3.7 Sonnet, Claude 3.5 Haiku, GPT-4o, o3-mini, DeepSeek-R1). Model pricing changes frequently across providers.

## 2. In-Memory Atomic Pricing Registry
- Pricing tables are represented as an immutable `Arc<HashMap<ModelIdentifier, ModelPricing>>`.
- When updated, a new map is loaded atomically using `ArcSwap`:
```rust
pub struct ModelPricing {
    pub prompt_cost_per_million: f64,
    pub completion_cost_per_million: f64,
    pub cache_read_cost_per_million: f64,
    pub cache_write_cost_per_million: f64,
}
```

## 3. Offline Embedded Fallback
In air-gapped or offline development environments, BurnRate falls back to an embedded compile-time pricing snapshot (`include_str!("../pricing.json")`), ensuring zero startup failures without internet access.
