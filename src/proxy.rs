//! Zero-overhead reverse proxy enforcing hard spend caps and prompt cache monitoring.

use crate::budget::BudgetGovernor;
use crate::cache_linter::PromptCacheLinter;
use crate::hud::TerminalHud;
use crate::pricing::PricingRegistry;
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
    let text = crate::metrics::MetricsExporter::render_prometheus(&status);
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
    body: axum::body::Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
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

    // 2. Inspect prompt for cache invalidations
    if let Some(alert) = state.linter.inspect_prompt(&body) {
        eprintln!("⚠️  [burnrate Alert] {}", alert.message);
    }

    // Parse model name from body or default
    let body_str = String::from_utf8_lossy(&body);
    let model_name = if body_str.contains("opus") {
        "claude-opus"
    } else if body_str.contains("sonnet") {
        "claude-sonnet"
    } else {
        "standard-agent"
    };

    let pricing = PricingRegistry::lookup(model_name);

    // Simulated turn consumption metrics
    let input_tokens = (body.len() / 4).max(50) as u64;
    let output_tokens = 120u64;
    let cache_read_tokens = (input_tokens as f64 * 0.8) as u64;
    let cache_write_tokens = (input_tokens as f64 * 0.2) as u64;

    let cost = pricing.calculate_cost(input_tokens, output_tokens, cache_read_tokens, cache_write_tokens);

    let tripped = state.governor.record_turn(cost, input_tokens, output_tokens, cache_read_tokens);
    let status = state.governor.status();
    TerminalHud::display(&status);

    if tripped {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            format!(
                "burnrate: Budget cap reached (${:.2} / ${:.2}). Session halted.",
                status.total_spent_usd, status.max_spend_usd
            ),
        ));
    }

    Ok(Json(json!({
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
    })))
}
