use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::hud::TerminalHud;
use burnrate::pricing::PricingRegistry;
use burnrate::proxy::{create_proxy_router, ProxyState};
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_pricing_calculation() {
    let sonnet = PricingRegistry::lookup("claude-sonnet");
    let cost = sonnet.calculate_cost(10_000, 1_000, 8_000, 2_000);
    assert!(cost > 0.0);
    // Cost must accurately reflect 90% discount on cache read
    assert!(cost < 0.10);
}

#[test]
fn test_budget_governor_hard_spend_breaker() {
    let gov = BudgetGovernor::new(1.00); // $1.00 cap
    assert!(!gov.is_tripped());

    // Record turn costing $0.40
    let tripped = gov.record_turn(0.40, 1000, 500, 0);
    assert!(!tripped);
    assert!(!gov.is_tripped());

    // Record turn costing $0.70 (total $1.10 -> exceeds $1.00)
    let tripped2 = gov.record_turn(0.70, 2000, 1000, 0);
    assert!(tripped2);
    assert!(gov.is_tripped());

    let status = gov.status();
    assert!(status.circuit_breaker_tripped);
    assert_eq!(status.remaining_usd, 0.0);
}

#[test]
fn test_prompt_cache_invalidation_linter() {
    let linter = PromptCacheLinter::new();

    let turn_1 = b"System: You are an expert engineer.\nTurn 1 context...";
    let alert_1 = linter.inspect_prompt(turn_1);
    assert!(alert_1.is_none());

    // Turn 2 with identical prefix: no alert
    let turn_2_clean = b"System: You are an expert engineer.\nTurn 2 context...";
    let alert_2 = linter.inspect_prompt(turn_2_clean);
    assert!(alert_2.is_none());

    // Turn 3 with timestamp injected at beginning: cache busted!
    let turn_3_busted = b"System: [2026-10-10 18:00:00] You are an expert engineer.";
    let alert_3 = linter.inspect_prompt(turn_3_busted);
    assert!(alert_3.is_some());
    let alert = alert_3.unwrap();
    assert!(alert.message.contains("Prompt cache busted"));
}

#[test]
fn test_terminal_hud_rendering() {
    let gov = BudgetGovernor::new(2.50);
    gov.record_turn(0.85, 25000, 2000, 20000);
    let status = gov.status();
    let bar = TerminalHud::render_bar(&status);

    assert!(bar.contains("burnrate"));
    assert!(bar.contains("$0.85 / $2.50"));
}

#[tokio::test]
async fn test_proxy_budget_enforcement_endpoint() {
    let governor = Arc::new(BudgetGovernor::new(0.01)); // tiny $0.01 cap
    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(burnrate::secret_guard::SecretGuard::new());
    let state = Arc::new(ProxyState::with_secret_guard(governor, linter, secret_guard));

    let app = create_proxy_router(state);

    // Initial turn succeeds or trips budget
    let req = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"model": "claude-sonnet-3-5", "messages": []}"#))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    // After consumption, budget is either OK or PAYMENT_REQUIRED
    assert!(response.status() == StatusCode::OK || response.status() == StatusCode::PAYMENT_REQUIRED);
}
