//! Zero-lag streaming token counter and chunk inspector for Server-Sent Events (SSE).

use std::sync::atomic::{AtomicU64, Ordering};

/// Streaming token counter that accumulates token chunks in real-time.
#[derive(Debug, Default)]
pub struct StreamingTokenCounter {
    pub total_input_bytes: AtomicU64,
    pub total_output_chunks: AtomicU64,
}

impl StreamingTokenCounter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest a stream chunk on the wire without buffering or adding latency.
    pub fn ingest_chunk(&self, chunk: &[u8]) {
        self.total_input_bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
        self.total_output_chunks.fetch_add(1, Ordering::Relaxed);
    }

    /// Approximate output token count from stream chunks (BPE average ~4 bytes/token).
    pub fn approximate_tokens(&self) -> u64 {
        let bytes = self.total_input_bytes.load(Ordering::Relaxed);
        (bytes / 4).max(1)
    }
}

/// Extract raw SSE data payloads without reallocating intermediate strings.
pub fn extract_sse_data_lines(raw_stream: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    for line in raw_stream.split(|&b| b == b'\n') {
        let trimmed = if line.ends_with(b"\r") {
            &line[..line.len() - 1]
        } else {
            line
        };
        if trimmed.starts_with(b"data: ") {
            lines.push(&trimmed[6..]);
        }
    }
    lines
}
