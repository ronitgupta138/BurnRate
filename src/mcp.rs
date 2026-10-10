//! Native Model Context Protocol (MCP) server for burnrate cost observability.

use crate::budget::BudgetGovernor;
use crate::hud::TerminalHud;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct McpServer {
    governor: Arc<BudgetGovernor>,
}

impl McpServer {
    pub fn new(max_spend: f64) -> Self {
        Self {
            governor: Arc::new(BudgetGovernor::new(max_spend)),
        }
    }

    pub fn run_loop<R: BufRead, W: Write>(&self, reader: R, mut writer: W) -> Result<(), anyhow::Error> {
        for line_res in reader.lines() {
            let line = line_res?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(trimmed) {
                let id = req.id.clone().unwrap_or(Value::Null);
                let is_notification = req.id.is_none();
                let resp_val = self.handle_request(&req);

                if !is_notification {
                    let resp = match resp_val {
                        Ok(res) => JsonRpcResponse {
                            jsonrpc: "2.0",
                            id,
                            result: Some(res),
                            error: None,
                        },
                        Err(err) => JsonRpcResponse {
                            jsonrpc: "2.0",
                            id,
                            result: None,
                            error: Some(json!({
                                "code": -32000,
                                "message": err.to_string()
                            })),
                        },
                    };

                    let out = serde_json::to_string(&resp)?;
                    writer.write_all(out.as_bytes())?;
                    writer.write_all(b"\n")?;
                    writer.flush()?;
                }
            }
        }
        Ok(())
    }

    pub fn handle_request(&self, req: &JsonRpcRequest) -> Result<Value, anyhow::Error> {
        match req.method.as_str() {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "burnrate",
                    "version": "0.2.0"
                }
            })),

            "notifications/initialized" => Ok(json!({})),
            "ping" => Ok(json!({})),

            "tools/list" => Ok(json!({
                "tools": self.list_tools()
            })),

            "tools/call" => {
                let name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = req.params.get("arguments").cloned().unwrap_or(json!({}));
                let output = self.call_tool(name, &args)?;
                Ok(json!({
                    "content": [
                        {
                            "type": "text",
                            "text": serde_json::to_string_pretty(&output)?
                        }
                    ]
                }))
            }

            other => Err(anyhow::anyhow!("Unsupported method: {}", other)),
        }
    }

    fn list_tools(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "burnrate_status",
                "description": "Inspect live session spend, remaining budget, burn rate ($/min), and cache hit ratio.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "burnrate_hud",
                "description": "Render formatted ANSI terminal status bar showing session FinOps health.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            })
        ]
    }

    pub fn call_tool(&self, name: &str, _args: &Value) -> Result<Value, anyhow::Error> {
        match name {
            "burnrate_status" => {
                let status = self.governor.status();
                Ok(serde_json::to_value(&status)?)
            }
            "burnrate_hud" => {
                let status = self.governor.status();
                let bar = TerminalHud::render_bar(&status);
                Ok(json!({ "hud": bar }))
            }
            unknown => Err(anyhow::anyhow!("Unknown tool: {}", unknown)),
        }
    }
}
