//! Automatic Anthropic Prompt-Cache breakpoint injector and optimizer.

use serde::{Deserialize, Serialize};

/// Status report after analyzing and optimizing prompt cache directives.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheOptimizationReport {
    pub breakpoint_injected: bool,
    pub target_location: Option<&'static str>,
    pub approximate_cached_tokens: u64,
}

pub struct CacheEngine {
    min_cache_threshold_chars: usize,
}

impl CacheEngine {
    pub fn new() -> Self {
        Self {
            min_cache_threshold_chars: 2_048, // Anthropic requires >= 1,024-2,048 tokens depending on model
        }
    }

    /// Automatically inspect Anthropic request payload and inject prompt-cache directives if absent.
    #[allow(clippy::collapsible_if)]
    pub fn optimize_anthropic_payload(
        &self,
        payload: &mut serde_json::Value,
    ) -> CacheOptimizationReport {
        let mut report = CacheOptimizationReport::default();

        let obj = match payload.as_object_mut() {
            Some(o) => o,
            None => return report,
        };

        // 1. Optimize `system` parameter
        if let Some(system_val) = obj.get_mut("system") {
            if let Some(system_str) = system_val.as_str() {
                if system_str.len() >= self.min_cache_threshold_chars {
                    let tokens = (system_str.len() / 4) as u64;
                    *system_val = serde_json::json!([
                        {
                            "type": "text",
                            "text": system_str,
                            "cache_control": { "type": "ephemeral" }
                        }
                    ]);
                    report.breakpoint_injected = true;
                    report.target_location = Some("system (string -> block)");
                    report.approximate_cached_tokens = tokens;
                    return report;
                }
            } else if let Some(system_blocks) = system_val.as_array_mut() {
                // If system is already an array of content blocks, check if cache_control is present
                let has_cache = system_blocks.iter().any(|b| b.get("cache_control").is_some());
                if !has_cache && !system_blocks.is_empty() {
                    let total_chars: usize = system_blocks
                        .iter()
                        .filter_map(|b| b.get("text").and_then(|t| t.as_str()).map(|s| s.len()))
                        .sum();

                    if total_chars >= self.min_cache_threshold_chars {
                        if let Some(last_block) = system_blocks.last_mut() {
                            if let Some(map) = last_block.as_object_mut() {
                                map.insert("cache_control".to_string(), serde_json::json!({ "type": "ephemeral" }));
                                report.breakpoint_injected = true;
                                report.target_location = Some("system (existing blocks)");
                                report.approximate_cached_tokens = (total_chars / 4) as u64;
                                return report;
                            }
                        }
                    }
                }
            }
        }

        // 2. Optimize tools definition if present
        if let Some(tools_val) = obj.get_mut("tools") {
            if let Some(tools_arr) = tools_val.as_array_mut() {
                let has_cache = tools_arr.iter().any(|t| t.get("cache_control").is_some());
                if !has_cache && !tools_arr.is_empty() {
                    let tools_chars: usize = serde_json::to_string(tools_arr).map(|s| s.len()).unwrap_or(0);
                    if tools_chars >= self.min_cache_threshold_chars {
                        if let Some(last_tool) = tools_arr.last_mut() {
                            if let Some(map) = last_tool.as_object_mut() {
                                map.insert("cache_control".to_string(), serde_json::json!({ "type": "ephemeral" }));
                                report.breakpoint_injected = true;
                                report.target_location = Some("tools");
                                report.approximate_cached_tokens = (tools_chars / 4) as u64;
                                return report;
                            }
                        }
                    }
                }
            }
        }

        // 3. Optimize early user messages if long static context is passed
        if let Some(messages_val) = obj.get_mut("messages") {
            if let Some(messages_arr) = messages_val.as_array_mut() {
                if let Some(first_msg) = messages_arr.first_mut() {
                    if let Some(content_str) = first_msg.get("content").and_then(|c| c.as_str()) {
                        if content_str.len() >= self.min_cache_threshold_chars {
                            let tokens = (content_str.len() / 4) as u64;
                            first_msg["content"] = serde_json::json!([
                                {
                                    "type": "text",
                                    "text": content_str,
                                    "cache_control": { "type": "ephemeral" }
                                }
                            ]);
                            report.breakpoint_injected = true;
                            report.target_location = Some("messages[0] (content string -> block)");
                            report.approximate_cached_tokens = tokens;
                            return report;
                        }
                    }
                }
            }
        }

        report
    }
}

impl Default for CacheEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_breakpoint_injection_on_large_system_prompt() {
        let engine = CacheEngine::new();

        // 1. Large string system prompt
        let large_system_text = "You are an expert systems engineer. ".repeat(100);
        assert!(large_system_text.len() > 3_000);

        let mut payload = serde_json::json!({
            "model": "claude-3-7-sonnet-20250219",
            "system": large_system_text,
            "messages": [
                { "role": "user", "content": "Hello" }
            ]
        });

        let report = engine.optimize_anthropic_payload(&mut payload);
        assert!(report.breakpoint_injected);
        assert_eq!(report.target_location, Some("system (string -> block)"));

        // Verify JSON now contains cache_control block
        let system_arr = payload["system"].as_array().expect("Converted to array");
        assert_eq!(system_arr.len(), 1);
        assert_eq!(system_arr[0]["cache_control"]["type"], "ephemeral");
    }

    #[test]
    fn test_cache_bypass_on_small_payload() {
        let engine = CacheEngine::new();
        let mut payload = serde_json::json!({
            "model": "claude-3-7-sonnet-20250219",
            "system": "You are a concise assistant.",
            "messages": [
                { "role": "user", "content": "Hi" }
            ]
        });

        let report = engine.optimize_anthropic_payload(&mut payload);
        assert!(!report.breakpoint_injected);
        assert!(payload["system"].is_string());
    }
}
