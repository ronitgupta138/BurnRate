use axum::body::Body;
use axum::http::{Request, StatusCode};
use burnrate::budget::BudgetGovernor;
use burnrate::cache_linter::PromptCacheLinter;
use burnrate::proxy::{create_proxy_router, ProxyState};
use burnrate::secret_guard::SecretGuard;
use burnrate::team::{AgentRole, TeamBudgetPool};
use std::sync::Arc;
use tower::ServiceExt;

#[test]
fn test_swarm_role_partitioning_and_summary() {
    let pool = TeamBudgetPool::new(10.00); // $10 total swarm pool

    // Allocate roles
    pool.allocate_agent_with_role("lead-architect", AgentRole::Architect, 5.00);
    pool.allocate_agent_with_role("worker-1", AgentRole::Worker, 3.00);
    pool.allocate_agent_with_role("tester-1", AgentRole::Tester, 1.50);
    pool.allocate_agent_with_role("reviewer-1", AgentRole::Reviewer, 0.50);

    let summary = pool.swarm_status();
    assert_eq!(summary.total_pool_usd, 10.00);
    assert_eq!(summary.registered_agents_count, 4);
    assert_eq!(summary.exhausted_agents_count, 0);

    // Record spend on worker-1
    let (worker_ex, pool_ex) = pool.record_spend_with_role("worker-1", AgentRole::Worker, 1.20);
    assert!(!worker_ex);
    assert!(!pool_ex);

    let quota = pool.get_agent_quota("worker-1").unwrap();
    assert_eq!(quota.spent_usd, 1.20);
    assert_eq!(quota.turns_count, 1);

    let table = pool.render_swarm_table();
    assert!(table.contains("lead-architect"));
    assert!(table.contains("worker-1"));
    assert!(table.contains("ACTIVE"));
}

#[tokio::test]
async fn test_proxy_multi_agent_swarm_header_enforcement() {
    let governor = Arc::new(BudgetGovernor::new(10.0));
    let linter = Arc::new(PromptCacheLinter::new());
    let secret_guard = Arc::new(SecretGuard::new());
    let state = Arc::new(ProxyState::with_secret_guard(governor, linter, secret_guard));

    let app = create_proxy_router(state.clone());

    // 1. Send request with swarm agent headers
    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [
            { "role": "user", "content": "Autonomous subagent turn 1" }
        ]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/v1/messages")
        .header("content-type", "application/json")
        .header("x-burnrate-agent-id", "subagent-coder-99")
        .header("x-burnrate-agent-role", "worker")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let remaining_hdr = response.headers().get("x-burnrate-agent-remaining");
    assert!(remaining_hdr.is_some(), "Expected x-burnrate-agent-remaining header");

    // Verify agent is enrolled in swarm pool
    let quota = state.swarm_pool.get_agent_quota("subagent-coder-99").unwrap();
    assert_eq!(quota.turns_count, 1);
    assert!(quota.spent_usd > 0.0);
}
