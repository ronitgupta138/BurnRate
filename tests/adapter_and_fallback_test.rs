use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::adapter::SchemaAdapter;
use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::fallback::LocalFallbackEngine;
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_schema_adapter_full_translation_loop() {
    // 1. Anthropic to OpenAI request
    let anthropic_req = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1024,
        "system": [
            { "type": "text", "text": "You are a database indexing expert." }
        ],
        "messages": [
            { "role": "user", "content": "How does B+ Tree split work?" }
        ]
    });

    let openai_req = SchemaAdapter::anthropic_to_openai(&anthropic_req);
    assert_eq!(openai_req["messages"][0]["role"], "system");
    assert_eq!(openai_req["messages"][0]["content"], "You are a database indexing expert.");
    assert_eq!(openai_req["messages"][1]["role"], "user");
    assert_eq!(openai_req["messages"][1]["content"], "How does B+ Tree split work?");

    // 2. OpenAI to Anthropic response
    let openai_resp = serde_json::json!({
        "id": "chatcmpl_test_42",
        "model": "gpt-4o",
        "choices": [
            {
                "index": 0,
                "message": { "role": "assistant", "content": "A B+ tree splits when a node exceeds M keys." },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 50,
            "completion_tokens": 25,
            "total_tokens": 75
        }
    });

    let anthropic_resp = SchemaAdapter::openai_to_anthropic_response(&openai_resp);
    assert_eq!(anthropic_resp["id"], "msg_adapted_chatcmpl_test_42");
    assert_eq!(anthropic_resp["type"], "message");
    assert_eq!(anthropic_resp["usage"]["input_tokens"], 50);
    assert_eq!(anthropic_resp["usage"]["output_tokens"], 25);
    assert_eq!(anthropic_resp["content"][0]["text"], "A B+ tree splits when a node exceeds M keys.");
}

#[test]
fn test_local_fallback_decision_logic() {
    let fallback = LocalFallbackEngine::new();

    // 1. Low spend ($0.50 / $2.00 = 25%) -> no fallback
    let dec1 = fallback.evaluate(0.50, 2.00, "claude-3-7-sonnet");
    assert!(!dec1.is_fallback);

    // 2. High spend ($1.91 / $2.00 = 95.5%) -> triggers fallback
    let dec2 = fallback.evaluate(1.91, 2.00, "claude-3-7-sonnet");
    assert!(dec2.is_fallback);
    assert_eq!(dec2.target_model, "deepseek-r1:latest");
    assert!(dec2.reason.contains("routed to local $0 offline model"));
}

#[tokio::test]
async fn test_proxy_fallback_header_when_spend_critical() {
    // Governor with $1.00 max spend and already consumed $0.96 (96%)
    let governor = Arc::new(BudgetGovernor::new(1.00));
    governor.record_turn(0.96, 5000, 100, 0);

    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(SecretGuard::new());
    let state = Arc::new(ProxyState::with_secret_guard(governor, linter, secret_guard));

    let app = create_proxy_router(state);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            { "role": "user", "content": "Write a quick test function." }
        ]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let fallback_hdr = response.headers().get("x-burnrate-fallback-active");
    assert!(fallback_hdr.is_some(), "Expected x-burnrate-fallback-active header");
}
