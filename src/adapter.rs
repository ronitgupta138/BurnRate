//! Bi-directional schema translator between Anthropic and OpenAI/DeepSeek LLM APIs.

use serde::{Deserialize, Serialize};

/// Target protocol for model request adaptation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetProtocol {
    Anthropic,
    OpenAi,
}

pub struct SchemaAdapter;

impl SchemaAdapter {
    /// Convert Anthropic `/v1/messages` request payload to OpenAI `/v1/chat/completions` payload.
    pub fn anthropic_to_openai(anthropic_body: &serde_json::Value) -> serde_json::Value {
        let mut openai_messages = Vec::new();

        // 1. Extract system prompt
        if let Some(system_val) = anthropic_body.get("system") {
            if let Some(sys_str) = system_val.as_str() {
                openai_messages.push(serde_json::json!({
                    "role": "system",
                    "content": sys_str
                }));
            } else if let Some(sys_arr) = system_val.as_array() {
                let combined_sys: String = sys_arr
                    .iter()
                    .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n\n");
                if !combined_sys.is_empty() {
                    openai_messages.push(serde_json::json!({
                        "role": "system",
                        "content": combined_sys
                    }));
                }
            }
        }

        // 2. Extract conversation turns
        if let Some(messages) = anthropic_body.get("messages").and_then(|m| m.as_array()) {
            for msg in messages {
                let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                let content_text = if let Some(content_str) = msg.get("content").and_then(|c| c.as_str()) {
                    content_str.to_string()
                } else if let Some(content_arr) = msg.get("content").and_then(|c| c.as_array()) {
                    content_arr
                        .iter()
                        .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    String::new()
                };

                openai_messages.push(serde_json::json!({
                    "role": role,
                    "content": content_text
                }));
            }
        }

        // 3. Map model identifier
        let anthropic_model = anthropic_body
            .get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("claude-3-7-sonnet-20250219");

        let mapped_model = if anthropic_model.contains("opus") {
            "gpt-4o"
        } else if anthropic_model.contains("haiku") {
            "gpt-4o-mini"
        } else if anthropic_model.contains("deepseek") {
            "deepseek-chat"
        } else {
            "gpt-4o"
        };

        let max_tokens = anthropic_body
            .get("max_tokens")
            .cloned()
            .unwrap_or(serde_json::json!(4096));

        let stream = anthropic_body
            .get("stream")
            .cloned()
            .unwrap_or(serde_json::json!(false));

        serde_json::json!({
            "model": mapped_model,
            "messages": openai_messages,
            "max_tokens": max_tokens,
            "stream": stream
        })
    }

    /// Convert OpenAI `/v1/chat/completions` response to Anthropic `/v1/messages` response.
    pub fn openai_to_anthropic_response(openai_resp: &serde_json::Value) -> serde_json::Value {
        let resp_id = openai_resp
            .get("id")
            .and_then(|id| id.as_str())
            .unwrap_or("chatcmpl_adapted");

        let content_text = openai_resp
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|first| first.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|txt| txt.as_str())
            .unwrap_or("");

        let prompt_tokens = openai_resp
            .get("usage")
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(|p| p.as_u64())
            .unwrap_or(0);

        let completion_tokens = openai_resp
            .get("usage")
            .and_then(|u| u.get("completion_tokens"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        let model_val = openai_resp
            .get("model")
            .cloned()
            .unwrap_or(serde_json::json!("adapted-model"));

        serde_json::json!({
            "id": format!("msg_adapted_{}", resp_id),
            "type": "message",
            "role": "assistant",
            "content": [
                {
                    "type": "text",
                    "text": content_text
                }
            ],
            "model": model_val,
            "stop_reason": "end_turn",
            "usage": {
                "input_tokens": prompt_tokens,
                "output_tokens": completion_tokens,
                "cache_read_input_tokens": 0,
                "cache_creation_input_tokens": 0
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_to_openai_translation() {
        let anthropic_req = serde_json::json!({
            "model": "claude-3-5-sonnet-20241022",
            "max_tokens": 2048,
            "system": "You are a Rust compiler specialist.",
            "messages": [
                { "role": "user", "content": "Explain phantom data." }
            ]
        });

        let openai_req = SchemaAdapter::anthropic_to_openai(&anthropic_req);
        let msgs = openai_req["messages"].as_array().expect("Array of messages");
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "system");
        assert_eq!(msgs[0]["content"], "You are a Rust compiler specialist.");
        assert_eq!(msgs[1]["role"], "user");
        assert_eq!(msgs[1]["content"], "Explain phantom data.");
    }

    #[test]
    fn test_openai_to_anthropic_response_translation() {
        let openai_resp = serde_json::json!({
            "id": "cmpl_9876",
            "model": "gpt-4o",
            "choices": [
                {
                    "index": 0,
                    "message": { "role": "assistant", "content": "PhantomData is zero-sized." },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 35,
                "completion_tokens": 12,
                "total_tokens": 47
            }
        });

        let anthropic_resp = SchemaAdapter::openai_to_anthropic_response(&openai_resp);
        assert_eq!(anthropic_resp["type"], "message");
        assert_eq!(anthropic_resp["role"], "assistant");
        assert_eq!(anthropic_resp["usage"]["input_tokens"], 35);
        assert_eq!(anthropic_resp["usage"]["output_tokens"], 12);
        assert_eq!(anthropic_resp["content"][0]["text"], "PhantomData is zero-sized.");
    }
}
