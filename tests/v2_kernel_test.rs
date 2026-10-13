use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::policy::{PolicyEngine, PolicyRule, PolicyVerdict};
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use burnrate::vault::TokenVault;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_policy_engine_custom_rules_and_verdict() {
    let mut policy = PolicyEngine::new(vec![
        PolicyRule::MaxInputTokens(50_000),
        PolicyRule::MaxSingleTurnCostAlert(0.25),
    ]);

    // 1. Normal turn -> Allowed
    let v1 = policy.evaluate("claude-sonnet", 10_000, 0.05, 0.50, 5.00);
    assert_eq!(v1, PolicyVerdict::Allowed);

    // 2. High single turn cost -> Warn
    let v2 = policy.evaluate("claude-sonnet", 10_000, 0.35, 0.50, 5.00);
    assert!(matches!(v2, PolicyVerdict::Warn(_)));

    // 3. Exceeded 50k tokens -> Denied
    let v3 = policy.evaluate("claude-sonnet", 55_000, 0.15, 0.50, 5.00);
    assert!(matches!(v3, PolicyVerdict::Denied(_)));

    // 4. Add rule to block opus when spend > 50%
    policy.add_rule(PolicyRule::DenyExpensiveModelOnHighSpend {
        model_substring: "opus".to_string(),
        spend_ratio_threshold: 0.50,
    });
    let v4 = policy.evaluate("claude-opus", 5_000, 0.10, 3.00, 5.00);
    assert!(matches!(v4, PolicyVerdict::Denied(_)));
}

#[test]
fn test_token_vault_key_injection() {
    let vault = TokenVault::new();
    vault.set_anthropic_key("sk-ant-test-vault-token-xyz");

    let mut headers = axum::http::HeaderMap::new();
    let injected = vault.inject_auth_headers(&mut headers, "claude-3-7-sonnet");
    assert!(injected);
    assert_eq!(headers.get("x-api-key").unwrap(), "sk-ant-test-vault-token-xyz");
}

#[tokio::test]
async fn test_proxy_v2_kernel_dashboard_and_policy_enforcement() {
    let governor = Arc::new(BudgetGovernor::new(10.0));
    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(SecretGuard::new());
    let state = Arc::new(ProxyState::with_secret_guard(governor, linter, secret_guard));

    let app = create_proxy_router(state);

    // 1. Verify /ui endpoint serves HTML dashboard
    let req_ui = Request::builder()
        .method("GET")
        .uri("/ui")
        .body(Body::empty())
        .unwrap();
    let resp_ui = app.clone().oneshot(req_ui).await.unwrap();
    assert_eq!(resp_ui.status(), StatusCode::OK);

    // 2. Verify /api/swarm endpoint
    let req_swarm = Request::builder()
        .method("GET")
        .uri("/api/swarm")
        .body(Body::empty())
        .unwrap();
    let resp_swarm = app.clone().oneshot(req_swarm).await.unwrap();
    assert_eq!(resp_swarm.status(), StatusCode::OK);

    // 3. Verify kernel version header on /v1/messages
    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            { "role": "user", "content": "Testing kernel v2.0.0" }
        ]
    });
    let req_msg = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let resp_msg = app.oneshot(req_msg).await.unwrap();
    assert_eq!(resp_msg.status(), StatusCode::OK);
    assert_eq!(
        resp_msg.headers().get("x-burnrate-kernel-version").unwrap(),
        "2.0.0"
    );
}
