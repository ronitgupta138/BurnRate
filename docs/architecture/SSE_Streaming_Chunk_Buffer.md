# Server-Sent Events (SSE) Streaming Chunk Buffer Zero-Copy Specification

## 1. Objective
BurnRate acts as an ultra-low-latency reverse proxy intercepting LLM completions (OpenAI, Anthropic, Ollama). When proxying token streams, it must count tokens and compute costs without introducing buffering latency or extra memory allocations.

## 2. Zero-Copy Byte Windowing
- Intercepts incoming HTTP/1.1 and HTTP/2 chunked transfer encoding streams via Tokio `AsyncRead`.
- Extracts token counts by scanning for `data: ` prefixes directly within raw byte slices (`&[u8]`) using SIMD-accelerated string scanning (`memchr`).
- Avoids allocating intermediate UTF-8 strings for streaming chunks that do not contain usage metadata.

## 3. Latency Guarantees
- Added proxy latency ceiling: $< 0.15\,\text{ms}$ per streaming chunk.
- Guarantees immediate flushing of downstream socket buffers so terminal interactive typing cadences remain uninterrupted.
