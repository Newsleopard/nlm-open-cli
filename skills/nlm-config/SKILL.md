---
name: nlm-config
version: 1.0.0
description: "nlm config: Manage API keys, profiles, and settings."
metadata:
  openclaw:
    category: "utility"
    domain: "config"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared"]
---

# nlm Config — Manage API Keys, Profiles, and Settings

Manage the `nlm` CLI configuration: API keys, output format preferences, MCP server URL,
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

# Set MCP server URL
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
| `NL_MCP_URL` | `mcp_url` | MCP server URL |

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

