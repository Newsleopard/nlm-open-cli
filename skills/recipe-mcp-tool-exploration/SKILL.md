---
name: recipe-mcp-tool-exploration
version: 1.0.0
description: "Discover and use MCP tools: list available tools, understand schemas, and invoke them."
metadata:
  openclaw:
    category: "recipe"
    domain: "mcp"
    requires:
      bins: ["nlm"]
      skills: ["nlm-mcp"]
---

# Recipe: MCP Tool Exploration

Discover and use MCP (Model Context Protocol) tools — list what is available,
inspect tool schemas, and invoke a tool with parameters.

## Prerequisites

- MCP server URL configured (`nlm config set mcp_url "https://..."`)
- The MCP server must be running and accessible

---

## Steps

### Step 1 — List available tools

Discover all tools exposed by the MCP server.

```bash
nlm mcp tools
```

This returns a JSON array of tool definitions, each with a `name`,
`description`, and `inputSchema`.

### Step 2 — Inspect a specific tool

Filter the tool list to examine one tool's schema in detail.

```bash
nlm mcp tools | jq '.[] | select(.name == "analyze_campaign")'
```

Review the `inputSchema` to understand required and optional parameters.

### Step 3 — Call a tool

Invoke the tool with a JSON argument payload.

```bash
nlm mcp call analyze_campaign --json '{"campaign_sn": "CAM12345"}'
```

The command sends a JSON-RPC 2.0 request to the MCP server and returns the
tool's response.

---

## Tips

- **Schema validation:** The CLI validates `--json` against the tool's
  `inputSchema` before sending the request. Missing required fields produce
  a validation error (exit code 2).
- **Output formatting:** MCP responses are JSON by default. Use `--format table`
  for a readable summary or pipe to `jq` for field extraction.
- **Tool discovery for agents:** AI agents can call `nlm mcp tools` to
  dynamically discover available capabilities — this is the primary use case
  for MCP integration.
- **Debugging:** Use `-vv` to see the full JSON-RPC request and response
  for troubleshooting.
