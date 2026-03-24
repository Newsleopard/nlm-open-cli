//! MCP and config skills: tool discovery via Model Context Protocol and configuration management.

use super::{SkillCategory, SkillDefinition};

pub fn skills() -> Vec<SkillDefinition> {
    vec![
        SkillDefinition {
            name: "nlm-mcp".to_string(),
            version: "1.0.0".to_string(),
            description: "nlm MCP: AI tool discovery and invocation via Model Context Protocol."
                .to_string(),
            category: SkillCategory::Utility,
            domain: Some("mcp".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string()],
            body: NLM_MCP.to_string(),
        },
        SkillDefinition {
            name: "nlm-config".to_string(),
            version: "1.0.0".to_string(),
            description: "nlm config: Manage API keys, profiles, and settings.".to_string(),
            category: SkillCategory::Utility,
            domain: Some("config".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string()],
            body: NLM_CONFIG.to_string(),
        },
    ]
}

const NLM_MCP: &str = r#"# nlm MCP — AI Tool Discovery and Invocation

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
"#;

const NLM_CONFIG: &str = r#"# nlm Config — Manage API Keys, Profiles, and Settings

Manage the `nlm` CLI configuration: API keys, output format preferences, MCP server base URL,
and multi-profile support for switching between environments.

**Config file location:** `~/.config/nl/config.toml` (permissions `0600`)

---

## Commands

### `nlm config init` — Interactive First-Time Setup

Launches an interactive wizard that guides you through:

1. **EDM API Key** — from the Newsleopard Dashboard (Settings > API Key)
2. **Surenotify API Key** — from the Surenotify Dashboard (Settings > API Key)
3. **Default output format** — choose from `json`, `table`, `yaml`, or `csv`

```bash
nlm config init
# Prompts:
#   EDM API Key: ********
#   Surenotify API Key: ********
#   Default format [json/table/yaml/csv]: table
#   Config saved to ~/.config/nl/config.toml
```

### `nlm config set KEY VALUE [--profile NAME]` — Set a Value

Set a configuration key. Without `--profile`, writes to the `[default]` section.

**Available keys:** `edm_api_key`, `sn_api_key`, `default_format`, `mcp_url`

```bash
# Set EDM API key in default profile
nlm config set edm_api_key "your-edm-key"

# Set Surenotify API key in a named profile
nlm config set sn_api_key "your-sn-key" --profile staging

# Set default output format
nlm config set default_format table

# Set MCP server base URL (`nlm` uses the `/mcp` endpoint)
nlm config set mcp_url "https://mcp.newsleopard.com"
```

### `nlm config get KEY [--profile NAME]` — Get a Value

Retrieve a config value. API keys are always masked with `****` in output.

```bash
nlm config get edm_api_key
# ****...

nlm config get default_format
# json

nlm config get sn_api_key --profile staging
# ****...
```

### `nlm config list` — Show All Settings

Display all configuration values. API keys are masked with `****`.

```bash
nlm config list
# [default]
# edm_api_key = ****...
# sn_api_key = ****...
# default_format = json
#
# [profiles.staging]
# edm_api_key = ****...
# sn_api_key = ****...
# default_format = table
```

---

## Profile Management

Profiles let you maintain separate configurations for different environments (e.g., staging vs. production) or different Newsleopard accounts.

### `nlm config profile create NAME` — Create a Profile

```bash
nlm config profile create staging
nlm config profile create production
```

### `nlm config profile list` — List All Profiles

```bash
nlm config profile list
# default
# staging
# production
```

### `nlm config profile delete NAME` — Delete a Profile

```bash
nlm config profile delete staging
```

---

## Multi-Profile Workflow

A common pattern is to maintain staging and production profiles with different API keys:

```bash
# 1. Create profiles
nlm config profile create staging
nlm config profile create production

# 2. Set API keys for each profile
nlm config set edm_api_key "staging-edm-key" --profile staging
nlm config set sn_api_key "staging-sn-key" --profile staging

nlm config set edm_api_key "prod-edm-key" --profile production
nlm config set sn_api_key "prod-sn-key" --profile production

# 3. Use a profile for a single command
nlm edm account balance --profile staging
nlm edm account balance --profile production

# 4. Set a profile for the entire shell session
export NL_PROFILE=staging
nlm edm contacts list          # uses staging profile
nlm sn email send --to a@b.com --subject "Test" --body "Hello"  # also staging

# 5. Override profile for one command
nlm edm account balance --profile production   # overrides NL_PROFILE
```

---

## Config File Structure

```toml
[default]
edm_api_key = "your-default-edm-key"
sn_api_key = "your-default-sn-key"
default_format = "json"
mcp_url = "https://mcp.newsleopard.com"

[profiles.staging]
edm_api_key = "staging-edm-key"
sn_api_key = "staging-sn-key"
default_format = "table"

[profiles.production]
edm_api_key = "prod-edm-key"
sn_api_key = "prod-sn-key"
```

---

## Environment Variable Overrides

Environment variables take the highest precedence and override both config file values and CLI flags:

| Variable | Config Key | Description |
|----------|------------|-------------|
| `NL_EDM_API_KEY` | `edm_api_key` | EDM API key |
| `NL_SN_API_KEY` | `sn_api_key` | Surenotify API key |
| `NL_FORMAT` | `default_format` | Output format |
| `NL_PROFILE` | (profile selector) | Active profile name |
| `NL_MCP_URL` | `mcp_url` | MCP server base URL (`nlm` uses the `/mcp` endpoint) |

```bash
# CI/CD pipeline example — no config file needed
export NL_EDM_API_KEY="ci-edm-key"
export NL_SN_API_KEY="ci-sn-key"
nlm edm contacts list
```

---

## Security

- **File permissions:** Config file is created with mode `0600` (owner read/write only). The CLI warns if permissions are looser.
- **Key masking:** API keys are never shown in plain text in `config list`, `config get`, or verbose/trace output.
- **No key logging:** Even at `-vv` verbosity, the `x-api-key` header value is replaced with `****`.
"#;
