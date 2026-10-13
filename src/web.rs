//! Embedded Zero-Dependency Web UI Dashboard for burnrate.

use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::Json;
use std::sync::Arc;

pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>burnrate // Autonomous Cost Kernel</title>
  <style>
    :root {
      --bg: #0a0b0e;
      --card-bg: #14171f;
      --border: #232838;
      --accent: #0891b2;
      --text: #f3f4f6;
      --text-muted: #9ca3af;
      --green: #10b981;
      --red: #ef4444;
      --yellow: #f59e0b;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace; }
    body { background: var(--bg); color: var(--text); padding: 24px; }
    .header { display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border); padding-bottom: 16px; margin-bottom: 24px; }
    .title { font-size: 20px; font-weight: 700; color: var(--accent); display: flex; align-items: center; gap: 8px; }
    .badge { background: #0891b222; border: 1px solid var(--accent); color: var(--accent); padding: 4px 10px; border-radius: 9999px; font-size: 12px; }
    .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 24px; }
    .card { background: var(--card-bg); border: 1px solid var(--border); border-radius: 8px; padding: 18px; }
    .card-label { font-size: 12px; text-transform: uppercase; color: var(--text-muted); margin-bottom: 8px; }
    .card-val { font-size: 24px; font-weight: 700; }
    .meter-container { background: #232838; border-radius: 999px; height: 10px; overflow: hidden; margin-top: 8px; }
    .meter-fill { height: 100%; background: var(--accent); width: 0%; transition: width 0.3s; }
    table { width: 100%; border-collapse: collapse; margin-top: 12px; font-size: 14px; }
    th { text-align: left; padding: 10px; color: var(--text-muted); border-bottom: 1px solid var(--border); }
    td { padding: 10px; border-bottom: 1px solid var(--border); }
    .active { color: var(--green); font-weight: 600; }
    .exhausted { color: var(--red); font-weight: 600; }
  </style>
</head>
<body>
  <div class="header">
    <div class="title">⚡ burnrate :: Autonomous Cost Kernel <span class="badge">v2.0.0</span></div>
    <div id="status-badge" class="badge">ONLINE</div>
  </div>

  <div class="grid">
    <div class="card">
      <div class="card-label">Session Spend</div>
      <div id="spend-val" class="card-val">$0.00 / $2.00</div>
      <div class="meter-container"><div id="meter-fill" class="meter-fill"></div></div>
    </div>
    <div class="card">
      <div class="card-label">Burn Rate</div>
      <div id="burn-val" class="card-val">$0.00 / min</div>
    </div>
    <div class="card">
      <div class="card-label">Prompt Cache Health</div>
      <div id="cache-val" class="card-val">0.0%</div>
    </div>
    <div class="card">
      <div class="card-label">Total Tokens Processed</div>
      <div id="tokens-val" class="card-val">0</div>
    </div>
  </div>

  <div class="card" style="margin-bottom: 24px;">
    <div class="card-label">Active Swarm Fleet Quotas</div>
    <table>
      <thead>
        <tr><th>Agent ID</th><th>Role</th><th>Spent</th><th>Allocated</th><th>Turns</th><th>Status</th></tr>
      </thead>
      <tbody id="swarm-tbody">
        <tr><td colspan="6" style="color: var(--text-muted);">No swarm workers active</td></tr>
      </tbody>
    </table>
  </div>

  <script>
    async function update() {
      try {
        const res = await fetch('/status');
        if (res.ok) {
          const data = await res.json();
          document.getElementById('spend-val').innerText = `$${data.total_spent_usd.toFixed(2)} / $${data.max_spend_usd.toFixed(2)}`;
          document.getElementById('burn-val').innerText = `$${data.burn_rate_usd_per_min.toFixed(2)} / min`;
          document.getElementById('cache-val').innerText = `${data.cache_hit_rate.toFixed(1)}%`;
          document.getElementById('tokens-val').innerText = data.total_tokens.toLocaleString();
          document.getElementById('meter-fill').style.width = `${Math.min(100, data.percent_consumed)}%`;
          if (data.circuit_breaker_tripped) {
            document.getElementById('status-badge').innerText = 'CIRCUIT BREAKER TRIPPED';
            document.getElementById('status-badge').style.borderColor = '#ef4444';
            document.getElementById('status-badge').style.color = '#ef4444';
          }
        }

        const swarmRes = await fetch('/api/swarm');
        if (swarmRes.ok) {
          const s = await swarmRes.json();
          const tbody = document.getElementById('swarm-tbody');
          if (s.agents && s.agents.length > 0) {
            tbody.innerHTML = s.agents.map(a => `
              <tr>
                <td><strong>${a.agent_id}</strong></td>
                <td>${a.role}</td>
                <td>$${a.spent_usd.toFixed(2)}</td>
                <td>$${a.allocated_budget_usd.toFixed(2)}</td>
                <td>${a.turns_count}</td>
                <td class="${a.is_exhausted ? 'exhausted' : 'active'}">${a.is_exhausted ? 'EXHAUSTED' : 'ACTIVE'}</td>
              </tr>
            `).join('');
          }
        }
      } catch (e) {
        console.error(e);
      }
    }
    setInterval(update, 2000);
    update();
  </script>
</body>
</html>
"#;

pub async fn ui_handler() -> impl IntoResponse {
    Html(DASHBOARD_HTML)
}

pub async fn api_swarm_handler(
    State(state): State<Arc<crate::proxy::ProxyState>>,
) -> impl IntoResponse {
    let summary = state.swarm_pool.swarm_status();
    Json(serde_json::json!({
        "summary": summary,
        "agents": state.swarm_pool.swarm_status()
    }))
}

pub async fn api_ledger_handler(
    State(state): State<Arc<crate::proxy::ProxyState>>,
) -> impl IntoResponse {
    let records = state
        .ledger
        .as_ref()
        .and_then(|l| l.list_records(20).ok())
        .unwrap_or_default();
    Json(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_dashboard_html_validity() {
        assert!(DASHBOARD_HTML.contains("burnrate :: Autonomous Cost Kernel"));
        assert!(DASHBOARD_HTML.contains("status-badge"));
        assert!(DASHBOARD_HTML.contains("/status"));
    }
}
