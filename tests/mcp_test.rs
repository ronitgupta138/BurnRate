use burnrate::mcp::McpServer;
use std::io::Cursor;

#[test]
fn test_mcp_initialize_and_tool_list() {
    let mcp = McpServer::new(2.00);
    let input = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}\n\
                 {\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{}}\n";

    let mut output = Vec::new();
    mcp.run_loop(Cursor::new(input), &mut output).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = output_str.trim().split('\n').collect();
    assert_eq!(lines.len(), 2);

    let init_res: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(init_res["result"]["serverInfo"]["name"], "burnrate");

    let tools_res: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    let tools = tools_res["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 2);
}

#[test]
fn test_mcp_call_status_tool() {
    let mcp = McpServer::new(2.50);
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "burnrate_status",
            "arguments": {}
        }
    });

    let mut output = Vec::new();
    mcp.run_loop(Cursor::new(serde_json::to_string(&req).unwrap() + "\n"), &mut output).unwrap();

    let res: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let content_text = res["result"]["content"][0]["text"].as_str().unwrap();
    let status_payload: serde_json::Value = serde_json::from_str(content_text).unwrap();

    assert_eq!(status_payload["max_spend_usd"], 2.5);
    assert_eq!(status_payload["circuit_breaker_tripped"], false);
}
