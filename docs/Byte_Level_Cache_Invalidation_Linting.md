# Byte-Level Prompt Prefix Cache Invalidation Mechanics

## 1. Problem Definition
When multiple AI agents or automated CI runners invoke LLMs, non-deterministic timestamps, UUIDs, or unordered JSON keys in the system prompt invalidate leading KV cache blocks. This forces the model provider to re-ingest the full prompt from scratch at full un-cached pricing.

## 2. Linting Algorithm
1. Extract prompt preamble up to the first user turn.
2. Canonicalize JSON tool definitions using deterministic lexicographical key sorting.
3. Detect dynamic volatile patterns (e.g. ISO 8601 timestamps, random session IDs) positioned within the first 1,024 tokens.
4. Flag violations with recommended refactoring: displace dynamic metadata to the user turn tail or tool context parameters.
