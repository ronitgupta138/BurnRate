use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::budget::BudgetGovernor;
use burnrate::cache_engine::CacheEngine;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_cache_engine_breakpoint_injection() {
    let engine = CacheEngine::new();
    let large_system = "You are an autonomous AI coding assistant specializing in systems programming. ".repeat(60);
    assert!(large_system.len() >= 2_048);

    let mut payload = serde_json::json!({
        "model": "claude-3-7-sonnet-20250219",
        "system": large_system,
        "messages": [
            { "role": "user", "content": "Analyze the codebase structure." }
        ]
    });

    let report = engine.optimize_anthropic_payload(&mut payload);
    assert!(report.breakpoint_injected);
    assert!(report.approximate_cached_tokens >= 500);

    let sys_array = payload["system"].as_array().expect("Array blocks");
    assert_eq!(sys_array[0]["cache_control"]["type"], "ephemeral");
}

#[tokio::test]
async fn test_proxy_end_to_end_context_truncation_and_cache_injection() {
    let governor = Arc::new(BudgetGovernor::new(10.0));
    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(SecretGuard::new());
    let state = Arc::new(ProxyState::with_secret_guard(governor, linter, secret_guard));

    let app = create_proxy_router(state);

    // Large system prompt + massive package-lock dump
    let large_system = "Static system prompt instructions for autonomous agent. ".repeat(60);
    let mut lockfile = String::from(r#"{"name": "test-app", "lockfileVersion": 3, "packages": {"#);
    for i in 0..150 {
        lockfile.push_str(&format!(r#""node_modules/lib-{}": {{"version": "2.0.{}"}}, "#, i, i));
    }
    lockfile.push_str(r#""ok": true}}"#);

    let payload = serde_json::json!({
        "model": "claude-3-7-sonnet-20250219",
        "system": large_system,
        "messages": [
            { "role": "user", "content": lockfile }
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

    // Check headers emitted by burnrate proxy
    let truncated_hdr = response.headers().get("x-burnrate-artifacts-truncated");
    assert!(truncated_hdr.is_some(), "Expected x-burnrate-artifacts-truncated header");

    let cache_hdr = response.headers().get("x-burnrate-cache-injected");
    assert!(cache_hdr.is_some(), "Expected x-burnrate-cache-injected header");
}
