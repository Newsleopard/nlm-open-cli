---
name: nlm-mcp
version: 1.0.0
description: "nlm MCP: AI tool discovery and invocation via Model Context Protocol."
metadata:
  openclaw:
    category: "utility"
    domain: "mcp"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared"]
---

# nlm MCP — AI Tool Discovery and Invocation

Discover and invoke MCP (Model Context Protocol) tools exposed by the Newsleopard platform.
MCP tools provide access to features not available via the REST API, such as AI-powered
campaign analysis, smart recommendations, audience insights, and content optimization.

The MCP client uses the **JSON-RPC 2.0** protocol to communicate with the server.

---

## Prerequisites

The MCP server base URL must be configured before using these commands:

```bash
# Set via config
nlm config set mcp_url "https://mcp.newsleopard.com"

# Or via environment variable
export NL_MCP_URL="https://mcp.newsleopard.com"
```

The default base URL is `https://mcp.newsleopard.com` if not explicitly configured. `nlm`
uses the JSON-RPC endpoint at `https://mcp.newsleopard.com/mcp`.

Published MCP endpoints:

- `https://mcp.newsleopard.com/mcp` — JSON-RPC endpoint used by `nlm mcp`
- `https://mcp.newsleopard.com/sse` — SSE endpoint for compatible MCP clients

---

## Commands

### `nlm mcp tools` — List Available Tools

Lists all available MCP tools with their descriptions and JSON parameter schemas.

```bash
# List all tools (default JSON output)
nlm mcp tools

# Table format for quick browsing
nlm mcp tools --format table

# Filter output with jq
nlm mcp tools | jq '.[].name'
```

**Example output (JSON):**

```json
[
  {
    "name": "analyze_campaign",
    "description": "AI-powered analysis of campaign performance with actionable recommendations",
    "parameters": {
      "type": "object",
      "properties": {
        "campaign_id": { "type": "integer", "description": "Campaign ID to analyze" },
        "depth": { "type": "string", "enum": ["summary", "detailed"], "default": "summary" }
      },
      "required": ["campaign_id"]
    }
  },
  {
    "name": "suggest_send_time",
    "description": "Recommend optimal send time based on audience engagement patterns",
    "parameters": {
      "type": "object",
      "properties": {
        "list_id": { "type": "integer", "description": "Mailing list ID" }
      },
      "required": ["list_id"]
    }
  }
]
```

### `nlm mcp call TOOL_NAME --json '{"key":"val"}'` — Invoke a Tool

Invoke any MCP tool by name, passing parameters as a JSON object.

```bash
# Analyze a campaign
nlm mcp call analyze_campaign --json '{"campaign_id": 12345}'

# Get detailed analysis
nlm mcp call analyze_campaign --json '{"campaign_id": 12345, "depth": "detailed"}'

# Suggest optimal send time for a mailing list
nlm mcp call suggest_send_time --json '{"list_id": 678}'

# Dry-run to preview the JSON-RPC request without sending
nlm mcp call analyze_campaign --json '{"campaign_id": 12345}' --dry-run
```

---

## Workflow: Discover Then Call

A typical MCP workflow is to discover available tools first, then invoke the desired one:

```bash
# Step 1: See what tools are available
nlm mcp tools --format table

# Step 2: Check a specific tool's parameter schema
nlm mcp tools | jq '.[] | select(.name == "analyze_campaign")'

# Step 3: Call the tool with the required parameters
nlm mcp call analyze_campaign --json '{"campaign_id": 12345}'
```

---

## Output Formats

MCP tool results respect the global `--format` flag:

```bash
# Pretty JSON (default)
nlm mcp call analyze_campaign --json '{"campaign_id": 12345}'

# Table view
nlm mcp call analyze_campaign --json '{"campaign_id": 12345}' --format table

# YAML
nlm mcp call analyze_campaign --json '{"campaign_id": 12345}' --format yaml
```

---

## Error Handling

MCP errors follow the standard JSON-RPC 2.0 error envelope and are mapped to `nlm` exit codes:

| Scenario | Exit Code | Error Type |
|----------|-----------|------------|
| Tool not found | 2 | Validation |
| Invalid parameters | 2 | Validation |
| Server unreachable | 4 | Network |
| MCP server error | 1 | Api |
| Invalid MCP URL | 3 | Config |

```bash
# Tool name typo — exit code 2
nlm mcp call nonexistent_tool --json '{}'
# stderr: {"error":{"type":"Validation","message":"MCP tool 'nonexistent_tool' not found","exit_code":2}}
```

---

## Notes

- The MCP client reuses the same retry and backoff logic as REST API calls (500ms initial, 30s max, 120s timeout).
- Tool discovery results can be cached locally — the tool list changes infrequently.
- All standard global flags (`--verbose`, `--quiet`, `--profile`, `--dry-run`) work with MCP commands.

