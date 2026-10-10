//! Prompt cache invalidation linter and byte-level prefix hash diff.

use std::sync::RwLock;

/// Warning issued when prompt cache invalidation is detected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CacheAlert {
    pub message: String,
    pub drift_byte_offset: usize,
    pub previous_prefix_hash: u64,
    pub current_prefix_hash: u64,
}

/// Linter tracking rolling prompt prefix hashes between agent turns.
pub struct PromptCacheLinter {
    last_prefix: RwLock<Option<(Vec<u8>, u64)>>,
}

impl PromptCacheLinter {
    pub fn new() -> Self {
        Self {
            last_prefix: RwLock::new(None),
        }
    }

    /// Inspect incoming turn prompt bytes. Detects cache busts at early byte offsets.
    pub fn inspect_prompt(&self, prompt_bytes: &[u8]) -> Option<CacheAlert> {
        // Evaluate first 32 bytes (common system prompt prefix)
        let prefix_len = prompt_bytes.len().min(32);
        let current_prefix = &prompt_bytes[..prefix_len];
        let current_hash = compute_hash(current_prefix);

        let mut lock = self.last_prefix.write().ok()?;

        if let Some((last_bytes, last_hash)) = lock.as_ref()
            && *last_hash != current_hash {
                // Find exact first diverging byte
                let mut drift_offset = 0;
                let min_len = last_bytes.len().min(current_prefix.len());
                while drift_offset < min_len && last_bytes[drift_offset] == current_prefix[drift_offset] {
                    drift_offset += 1;
                }

                let alert = CacheAlert {
                    message: format!(
                        "Prompt cache busted at byte {} (prefix drift detected)",
                        drift_offset
                    ),
                    drift_byte_offset: drift_offset,
                    previous_prefix_hash: *last_hash,
                    current_prefix_hash: current_hash,
                };

                *lock = Some((current_prefix.to_vec(), current_hash));
                return Some(alert);
        }

        *lock = Some((current_prefix.to_vec(), current_hash));
        None
    }
}

impl Default for PromptCacheLinter {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64; // FNV-1a 64-bit offset basis
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
