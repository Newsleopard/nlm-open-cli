# nl-cli

**[繁體中文](README.zh-TW.md)** | English

**One CLI for NewsLeopard EDM and SureNotify APIs — built for developers and AI agents.**

Manage email campaigns, transactional messages, contacts, templates, and reports from a single command-line tool. Structured JSON output. Built-in rate limiting. Dry-run safety.

[![CI](https://github.com/newsleopard/nl-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/newsleopard/nl-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/newsleopard/nl-cli)](https://github.com/newsleopard/nl-cli/releases)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust: 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Why nl?](#why-nl)
- [Commands](#commands)
- [Authentication](#authentication)
- [Global Flags](#global-flags)
- [Programmatic Usage](#programmatic-usage)
- [Variable Syntax](#variable-syntax)
- [Rate Limits](#rate-limits)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+ (for building from source) or download a pre-built binary
- A [NewsLeopard](https://www.newsleopard.com/) account with an EDM or SureNotify API key

## Installation

### npm

```bash
npm install -g @newsleopard/nl-cli
```

### Cargo

```bash
cargo install nl-cli
```

### GitHub Releases

Download pre-built binaries for your platform from [Releases](https://github.com/newsleopard/nl-cli/releases).

Available targets: Linux (x86_64, arm64), macOS (x86_64, arm64), Windows (x86_64).

### Build from Source

```bash
git clone https://github.com/newsleopard/nl-cli.git
cd nl-cli
cargo build --release
# Binary at target/release/nl
```

## Quick Start

```bash
# 1. Configure your API key
nl config init

# 2. Check account balance
nl edm account balance

# 3. List contact groups
nl edm contacts list-groups
```

## Why nl?

### Before: raw API calls

```bash
curl -s -H "x-api-key: $KEY" \
  "https://api.newsleopard.com/v1/contacts/groups?page=1&per_page=20" \
  | jq '.data'
```

### After: one command

```bash
nl edm contacts list-groups --format table
```

**Key benefits:**

- **34 API endpoints** wrapped in intuitive subcommands — no URL/header juggling
- **Structured output** — JSON, Table, YAML, CSV; JSON auto-compacts when piped
- **Built-in rate limiting** — token bucket respects API limits (2 req/s EDM, 1 req/10s report export)
- **Smart retry** — exponential backoff on 429 and 5xx errors
- **Dry-run mode** — preview HTTP requests without sending (`--dry-run`)
- **Helper workflows** — orchestrate multi-step operations like `campaign-send` and `import-and-wait`
- **Multi-profile config** — switch between staging and production with `--profile`

## Commands

| Group | Description | Endpoints |
|-------|-------------|-----------|
| `nl edm contacts` | Contact group management (create, list, import, delete) | 6 |
| `nl edm campaign` | Email campaign management (submit, status, pause, delete) | 5 |
| `nl edm ab-test` | A/B test campaigns | 2 |
| `nl edm report` | Campaign reports (list, metrics, export, download) | 4 |
| `nl edm template` | Template management (list, get) | 2 |
| `nl edm automation` | Automation script triggers | 1 |
| `nl edm account` | Account info (balance) | 1 |
| `nl sn email` | Transactional email (send, query events) | 2 |
| `nl sn sms` | SMS (send, query events, dedicated numbers) | 3 |
| `nl sn webhook` | Email webhook CRUD | 3 |
| `nl sn sms-webhook` | SMS webhook CRUD | 3 |
| `nl sn domain` | Sender domain verification (create, verify, remove) | 3 |
| `nl mcp` | MCP tool discovery and invocation (for AI agents) | 2 |
| `nl config` | Config file management | -- |
| `nl helper` | High-level orchestration workflows | -- |

## Authentication

| Scenario | Method | Setup |
|----------|--------|-------|
| Interactive (local dev) | Config file | `nl config init` |
| CI/CD or containers | Environment variables | `export NL_EDM_API_KEY="..."` |
| Multiple environments | Profiles | `nl config set edm_api_key "..." --profile staging` |

**Credential precedence:** Environment variable > CLI flag > Profile config > `[default]` section.

### Config File

Located at `~/.config/nl/config.toml`:

```toml
[default]
edm_api_key = "your-edm-key"
sn_api_key = "your-sn-key"
default_format = "json"

[staging]
edm_api_key = "staging-key"
sn_api_key = "staging-sn-key"
```

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `NL_EDM_API_KEY` | EDM API key |
| `NL_SN_API_KEY` | SureNotify API key |
| `NL_PROFILE` | Active profile name (default: `default`) |
| `NL_FORMAT` | Default output format (default: `json`) |
| `NL_MCP_URL` | MCP server URL (default: `https://mcp.newsleopard.com`) |

## Global Flags

```
--format <json|table|yaml|csv>   Output format (default: json, env: NL_FORMAT)
--profile <NAME>                 Config profile (default: default, env: NL_PROFILE)
--dry-run                        Preview request without executing
--page-all                       Stream paginated results as NDJSON
-v, --verbose                    Show request/response details (stackable: -vv)
-q, --quiet                      Errors only
```

> **Piping behavior:** `--format json` outputs pretty-printed JSON in a terminal, compact JSON when piped. `--page-all` streams NDJSON (one JSON object per line), suitable for `jq` line-by-line processing.

## Programmatic Usage

The CLI is designed for scripting and AI agent integration. All output is machine-parseable with structured exit codes and JSON error output on stderr.

### Exit Codes

| Code | Meaning | Trigger |
|------|---------|---------|
| 0 | Success | Normal response, dry-run preview, 204 No Content |
| 1 | API error | HTTP 4xx/5xx from NewsLeopard API |
| 2 | Validation error | CLI argument validation failed |
| 3 | Auth/config error | Invalid API key, missing or corrupt config |
| 4 | Network/rate limit | Connection failure, daily quota exhausted |
| 5 | I/O error | File read/write failure |

### Error Output

All errors are JSON on **stderr** with a `type` field (`api`, `validation`, `auth`, `config`, `network`, `rate_limit`, `io`):

```json
{
  "error": {
    "type": "api",
    "message": "API error 400: [40012] Insufficient balance",
    "exit_code": 1
  }
}
```

### Scripting Examples

```bash
# Get campaign open rate
result=$(nl edm report metrics --campaign-sn "$SN" -q 2>/tmp/nl_err.json)
if [ $? -eq 0 ]; then
  echo "$result" | jq '.open_rate'
else
  echo "Failed: $(jq -r '.error.type' /tmp/nl_err.json)" >&2
fi
```

```bash
# Stream all groups, filter by open rate > 30%
nl edm contacts list-groups --page-all -q | jq 'select(.opened_rate > 0.3)'
```

```bash
# Dry-run to preview a campaign submit request
nl edm campaign submit --name "March Newsletter" --dry-run
```

## Variable Syntax

EDM and SureNotify use **different variable syntaxes**. Mixing them causes silent substitution failures.

| API | Syntax | Example | Used in |
|-----|--------|---------|---------|
| EDM | `${FIELD_NAME}` | `${NAME}`, `${ORDER_ID}` | `nl edm campaign`, `nl edm ab-test`, `nl edm automation` |
| SureNotify | `{{variable_name}}` | `{{name}}`, `{{order_id}}` | `nl sn email`, `nl sn sms` |

> The CLI detects and warns on cross-use (e.g., `{{...}}` in EDM content).

## Rate Limits

Built-in token bucket rate limiting ensures API compliance automatically:

| Limit | Value | Affected Commands |
|-------|-------|-------------------|
| EDM general | 2 req/s | All `nl edm` commands |
| Report export | 1 req/10s | `nl edm report export` |
| SN recipients | 100 per request | `nl sn email send`, `nl sn sms send` |

HTTP 429 and 5xx errors are retried with exponential backoff (500ms initial, 30s max, 120s total timeout).

## Documentation

- [CLI User Guide](docs/CLI-USER-GUIDE.md) — Complete command tree with examples
- [NewsLeopard API Agent Skill](https://github.com/Newsleopard/nlm-open-skills) — AI agent skill for generating NewsLeopard API integration code (supports Claude Code, GitHub Copilot, Cursor)

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code style, and PR guidelines.

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

## License

Licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
