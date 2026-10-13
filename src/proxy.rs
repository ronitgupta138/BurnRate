//! Zero-overhead reverse proxy enforcing hard spend caps and prompt cache monitoring.

use crate::budget::BudgetGovernor;
use crate::cache_linter::PromptCacheLinter;
use crate::hud::TerminalHud;
use crate::pricing::PricingRegistry;
use crate::secret_guard::SecretGuard;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub struct ProxyState {
    pub governor: Arc<BudgetGovernor>,
    pub linter: Arc<PromptCacheLinter>,
    pub secret_guard: Arc<SecretGuard>,
    pub ledger: Option<Arc<crate::ledger::Ledger>>,
    pub context_guard: Arc<crate::context_guard::ContextWindowGuard>,
    pub cache_engine: Arc<crate::cache_engine::CacheEngine>,
    pub fallback_engine: Arc<crate::fallback::LocalFallbackEngine>,
    pub swarm_pool: Arc<crate::team::TeamBudgetPool>,
}

impl ProxyState {
    pub fn new(governor: Arc<BudgetGovernor>, linter: Arc<PromptCacheLinter>) -> Self {
        let max_pool = governor.status().max_spend_usd;
        Self {
            governor,
            linter,
            secret_guard: Arc::new(SecretGuard::new()),
            ledger: None,
            context_guard: Arc::new(crate::context_guard::ContextWindowGuard::default()),
            cache_engine: Arc::new(crate::cache_engine::CacheEngine::default()),
            fallback_engine: Arc::new(crate::fallback::LocalFallbackEngine::default()),
            swarm_pool: Arc::new(crate::team::TeamBudgetPool::new(max_pool)),
        }
    }

    pub fn with_secret_guard(
        governor: Arc<BudgetGovernor>,
        linter: Arc<PromptCacheLinter>,
        secret_guard: Arc<SecretGuard>,
    ) -> Self {
        let max_pool = governor.status().max_spend_usd;
        Self {
            governor,
            linter,
            secret_guard,
            ledger: None,
            context_guard: Arc::new(crate::context_guard::ContextWindowGuard::default()),
            cache_engine: Arc::new(crate::cache_engine::CacheEngine::default()),
            fallback_engine: Arc::new(crate::fallback::LocalFallbackEngine::default()),
            swarm_pool: Arc::new(crate::team::TeamBudgetPool::new(max_pool)),
        }
    }

    pub fn with_ledger(
        governor: Arc<BudgetGovernor>,
        linter: Arc<PromptCacheLinter>,
        secret_guard: Arc<SecretGuard>,
        ledger: Arc<crate::ledger::Ledger>,
    ) -> Self {
        let max_pool = governor.status().max_spend_usd;
        Self {
            governor,
            linter,
            secret_guard,
            ledger: Some(ledger),
            context_guard: Arc::new(crate::context_guard::ContextWindowGuard::default()),
            cache_engine: Arc::new(crate::cache_engine::CacheEngine::default()),
            fallback_engine: Arc::new(crate::fallback::LocalFallbackEngine::default()),
            swarm_pool: Arc::new(crate::team::TeamBudgetPool::new(max_pool)),
        }
    }
}

pub fn create_proxy_router(state: Arc<ProxyState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .route("/metrics", get(metrics_handler))
        .route("/v1/messages", post(proxy_messages_handler))
        .route("/v1/chat/completions", post(proxy_messages_handler))
        .layer(cors)
        .with_state(state)
}

async fn metrics_handler(State(state): State<Arc<ProxyState>>) -> impl IntoResponse {
    let status = state.governor.status();
    let text = crate::metrics::MetricsExporter::render_swarm_prometheus(&status, &state.swarm_pool);
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4")],
        text,
    )
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "burnrate",
        "version": "1.0.0"
    }))
}

async fn status_handler(State(state): State<Arc<ProxyState>>) -> impl IntoResponse {
    Json(state.governor.status())
}

async fn proxy_messages_handler(
    State(state): State<Arc<ProxyState>>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let agent_id_opt = headers
        .get("x-burnrate-agent-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let agent_role = headers
        .get("x-burnrate-agent-role")
        .and_then(|h| h.to_str().ok())
        .map(crate::team::AgentRole::parse)
        .unwrap_or(crate::team::AgentRole::Worker);

    if let Some(ref agent_id) = agent_id_opt {
        let q = state.swarm_pool.enroll_or_get(agent_id, agent_role);
        if q.is_exhausted {
            return Err((
                StatusCode::PAYMENT_REQUIRED,
                format!(
                    "burnrate: Swarm agent `{}` quota exhausted (${:.2} / ${:.2}). Execution paused.",
                    agent_id, q.spent_usd, q.allocated_budget_usd
                ),
            ));
        }
    }

    // 1. Check if budget already tripped
    if state.governor.is_tripped() {
        let status = state.governor.status();
        TerminalHud::display(&status);
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            format!(
                "burnrate: Budget cap reached (${:.2} / ${:.2}). Session halted.",
                status.total_spent_usd, status.max_spend_usd
            ),
        ));
    }

    // Context optimization: truncate oversized artifacts (lockfiles, base64) and inject prompt-cache directives
    let (body_after_context_opt, artifacts_truncated, cache_injected) = {
        if let Ok(mut json_val) = serde_json::from_slice::<serde_json::Value>(&body) {
            let trunc_stats = state.context_guard.sanitize_json(&mut json_val);
            if trunc_stats.count_truncated > 0 {
                eprintln!(
                    "⚠️ [burnrate] Truncated {} oversized artifact(s), saved ~{} tokens",
                    trunc_stats.count_truncated, trunc_stats.estimated_tokens_saved
                );
            }
            let cache_report = state.cache_engine.optimize_anthropic_payload(&mut json_val);
            if cache_report.breakpoint_injected {
                eprintln!(
                    "⚡ [burnrate] Injected prompt-cache breakpoint at {:?} (~{} cached tokens)",
                    cache_report.target_location.unwrap_or("prompt"), cache_report.approximate_cached_tokens
                );
            }
            if trunc_stats.count_truncated > 0 || cache_report.breakpoint_injected {
                let bytes = serde_json::to_vec(&json_val).unwrap_or_else(|_| body.to_vec());
                (axum::body::Bytes::from(bytes), trunc_stats.count_truncated, cache_report.breakpoint_injected)
            } else {
                (body, 0, false)
            }
        } else {
            (body, 0, false)
        }
    };

    // Intercept and redact secrets from outbound prompt payload
    let (effective_body, body_str, redacted_count) = {
        let (sanitized_body_str, count) = {
            let raw_body_str = String::from_utf8_lossy(&body_after_context_opt);
            state.secret_guard.redact(&raw_body_str)
        };
        if count > 0 {
            eprintln!(
                "⚠️ [burnrate] Intercepted and redacted {} secret(s) from outbound prompt payload",
                count
            );
            (axum::body::Bytes::from(sanitized_body_str.clone()), sanitized_body_str, count)
        } else {
            let owned_str = String::from_utf8_lossy(&body_after_context_opt).into_owned();
            (body_after_context_opt, owned_str, 0)
        }
    };

    // 2. Inspect prompt for cache invalidations
    if let Some(alert) = state.linter.inspect_prompt(&effective_body) {
        eprintln!("⚠️  [burnrate Alert] {}", alert.message);
    }

    // Parse model name from body or default
    let model_name = if body_str.contains("opus") {
        "claude-opus"
    } else if body_str.contains("sonnet") {
        "claude-sonnet"
    } else {
        "standard-agent"
    };

    let status_pre = state.governor.status();
    let fallback = state.fallback_engine.evaluate(
        status_pre.total_spent_usd,
        status_pre.max_spend_usd,
        model_name,
    );
    if fallback.is_fallback {
        eprintln!("🔄 [burnrate Fallback] {}", fallback.reason);
    }

    let pricing = PricingRegistry::lookup(model_name);

    // Simulated turn consumption metrics
    let input_tokens = (effective_body.len() / 4).max(50) as u64;
    let output_tokens = 120u64;
    let cache_read_tokens = (input_tokens as f64 * 0.8) as u64;
    let cache_write_tokens = (input_tokens as f64 * 0.2) as u64;

    let cost = if fallback.is_fallback {
        0.0
    } else {
        pricing.calculate_cost(input_tokens, output_tokens, cache_read_tokens, cache_write_tokens)
    };

    let mut swarm_exhausted = false;
    if let Some(ref agent_id) = agent_id_opt {
        let (agent_ex, pool_ex) = state.swarm_pool.record_spend_with_role(agent_id, agent_role, cost);
        if agent_ex || pool_ex {
            swarm_exhausted = true;
        }
    }

    let tripped = state.governor.record_turn(cost, input_tokens, output_tokens, cache_read_tokens);
    let status = state.governor.status();
    TerminalHud::display(&status);

    if let Some(ledger) = &state.ledger {
        let record = crate::ledger::SessionRecord::new(
            model_name,
            cost,
            status.max_spend_usd,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            redacted_count,
            tripped || swarm_exhausted,
        );
        let _ = ledger.record_session(&record);
    }

    if tripped || swarm_exhausted {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            format!(
                "burnrate: Budget cap reached (${:.2} / ${:.2}). Session halted.",
                status.total_spent_usd, status.max_spend_usd
            ),
        ));
    }

    let mut response_headers = axum::http::HeaderMap::new();
    if redacted_count > 0 {
        response_headers.insert(
            axum::http::HeaderName::from_static("x-burnrate-secrets-redacted"),
            axum::http::HeaderValue::from(redacted_count),
        );
    }
    if artifacts_truncated > 0 {
        response_headers.insert(
            axum::http::HeaderName::from_static("x-burnrate-artifacts-truncated"),
            axum::http::HeaderValue::from(artifacts_truncated),
        );
    }
    if cache_injected {
        response_headers.insert(
            axum::http::HeaderName::from_static("x-burnrate-cache-injected"),
            axum::http::HeaderValue::from_static("true"),
        );
    }
    if fallback.is_fallback {
        response_headers.insert(
            axum::http::HeaderName::from_static("x-burnrate-fallback-active"),
            axum::http::HeaderValue::from_static("true"),
        );
    }
    #[allow(clippy::collapsible_if)]
    if let Some(ref agent_id) = agent_id_opt {
        if let Some(q) = state.swarm_pool.get_agent_quota(agent_id) {
            let rem = (q.allocated_budget_usd - q.spent_usd).max(0.0);
            if let Ok(val) = axum::http::HeaderValue::from_str(&format!("{:.4}", rem)) {
                response_headers.insert(
                    axum::http::HeaderName::from_static("x-burnrate-agent-remaining"),
                    val,
                );
            }
        }
    }

    Ok((
        response_headers,
        Json(json!({
            "id": "msg_simulated",
            "type": "message",
            "role": "assistant",
            "content": [{ "type": "text", "text": "burnrate proxy forward simulated response" }],
            "usage": {
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cache_read_input_tokens": cache_read_tokens,
                "cache_creation_input_tokens": cache_write_tokens
            }
        })),
    ))
}
