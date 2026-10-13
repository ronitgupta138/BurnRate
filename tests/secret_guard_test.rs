use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_secret_guard_all_credential_types() {
    let guard = SecretGuard::new();

    // 1. Anthropic Key
    let (res, count) = guard.redact("My key is sk-ant-api03-testkey1234567890abcdef here");
    assert_eq!(count, 1);
    assert!(res.contains("[REDACTED_BY_BURNRATE:ANTHROPIC_KEY]"));
    assert!(!res.contains("sk-ant-api03-testkey1234567890abcdef"));

    // 2. OpenAI Key
    let (res, count) = guard.redact("OpenAI sk-proj-abcdef1234567890abcdef1234567890 key");
    assert_eq!(count, 1);
    assert!(res.contains("[REDACTED_BY_BURNRATE:OPENAI_KEY]"));

    // 3. AWS Key
    let (res, count) = guard.redact("AWS credentials: AKIAIOSFODNN7EXAMPLE");
    assert_eq!(count, 1);
    assert!(res.contains("[REDACTED_BY_BURNRATE:AWS_ACCESS_KEY_ID]"));

    // 4. GitHub PAT
    let (res, count) = guard.redact("Token: ghp_123456789012345678901234567890123456");
    assert_eq!(count, 1);
    assert!(res.contains("[REDACTED_BY_BURNRATE:GITHUB_PAT]"));

    // 5. PEM Private Key
    let (res, count) = guard.redact("Certificate: -----BEGIN RSA PRIVATE KEY-----\nMIIEow...");
    assert_eq!(count, 1);
    assert!(res.contains("[REDACTED_BY_BURNRATE:PEM_PRIVATE_KEY]"));

    // 6. Clean Text Unaltered
    let (clean, count) = guard.redact("Clean safe prompt text without any secret tokens.");
    assert_eq!(count, 0);
    assert_eq!(clean, "Clean safe prompt text without any secret tokens.");
}

#[tokio::test]
async fn test_proxy_secret_redaction_header_and_body() {
    let governor = Arc::new(BudgetGovernor::new(10.0));
    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(SecretGuard::new());
    let state = Arc::new(ProxyState {
        governor,
        linter,
        secret_guard,
    });

    let app = create_proxy_router(state);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            { "role": "user", "content": "Help me fix my code with key sk-ant-api03-12345678901234567890" }
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
    let redacted_header = response.headers().get("x-burnrate-secrets-redacted");
    assert!(redacted_header.is_some());
    assert_eq!(redacted_header.unwrap(), "1");
}
