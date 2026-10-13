use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::config::BurnrateConfig;
use burnrate::ledger::{Ledger, SessionRecord};
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_ledger_record_and_summary_lifecycle() {
    let tmp = std::env::temp_dir().join(format!("test_ledger_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(42)));
    let file = tmp.join("test.jsonl");
    let ledger = Ledger::new(&file);

    assert_eq!(ledger.list_records(10).unwrap().len(), 0);

    let rec1 = SessionRecord::new("claude-sonnet", 0.50, 2.00, 10000, 500, 8000, 0, false);
    let rec2 = SessionRecord::new("claude-opus", 2.10, 2.00, 20000, 1000, 15000, 2, true);

    ledger.record_session(&rec1).unwrap();
    ledger.record_session(&rec2).unwrap();

    let records = ledger.list_records(10).unwrap();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].model, "claude-opus"); // newest first

    let summary = ledger.summary().unwrap();
    assert_eq!(summary.total_sessions, 2);
    assert!((summary.total_spent_usd - 2.60).abs() < 0.01);
    assert_eq!(summary.total_secrets_redacted, 2);
    assert_eq!(summary.tripped_sessions, 1);

    let table = Ledger::render_table(&records);
    assert!(table.contains("claude-opus"));
    assert!(table.contains("TRIPPED"));

    let _ = std::fs::remove_dir_all(tmp);
}

#[tokio::test]
async fn test_proxy_integration_records_to_ledger() {
    let tmp = std::env::temp_dir().join(format!("test_proxy_ledger_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(99)));
    let file = tmp.join("proxy_ledger.jsonl");
    let ledger = Arc::new(Ledger::new(&file));

    let governor = Arc::new(BudgetGovernor::new(5.0));
    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(SecretGuard::new());

    let state = Arc::new(ProxyState::with_ledger(
        governor,
        linter,
        secret_guard,
        ledger.clone(),
    ));

    let app = create_proxy_router(state);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            { "role": "user", "content": "Write a rust function for binary search." }
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

    // Verify ledger has recorded the session
    let records = ledger.list_records(10).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].model, "claude-sonnet");
    assert!(records[0].total_spent_usd > 0.0);

    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn test_config_load_and_serialization() {
    let cfg = BurnrateConfig {
        default_max_spend: 3.50,
        default_port: 8080,
        ..Default::default()
    };

    let json = serde_json::to_string(&cfg).unwrap();
    let deserialized: BurnrateConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.default_max_spend, 3.50);
    assert_eq!(deserialized.default_port, 8080);
}
