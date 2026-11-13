# Model Context Protocol (MCP) Stdio JSON-RPC Bridge Specification

## 1. Architecture
BurnRate functions as an MCP middleware bridge, inspecting and accounting tool execution costs when coding agents invoke external MCP server tools.

## 2. Frame Framing & Serialization
- Communicates over standard input/output (`stdin`/`stdout`) using newline-delimited or Content-Length framed JSON-RPC 2.0 messages.
- Parses `tools/call` requests and matches response payloads against output schema token boundaries.

## 3. Child Process Superversion
BurnRate monitors spawned child MCP processes. If a downstream MCP tool server crashes or hangs, BurnRate captures the error envelope and issues an RFC 7807 error without terminating the parent Claude Code session.
