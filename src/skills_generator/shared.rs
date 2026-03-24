//! Shared skill: cross-cutting concerns (auth, flags, formats, errors, rate limits).

use super::{SkillCategory, SkillDefinition};

pub fn skills() -> Vec<SkillDefinition> {
    vec![SkillDefinition {
        name: "nlm-shared".to_string(),
        version: "1.0.0".to_string(),
        description: "nlm CLI: Authentication, global flags, output formats, error codes, variable syntax, and rate limits.".to_string(),
        category: SkillCategory::Shared,
        domain: None,
        requires_bins: vec!["nlm".to_string()],
        requires_skills: vec![],
        body: NLM_SHARED.to_string(),
    }]
}

const NLM_SHARED: &str = r#"# nlm — Shared Reference

Cross-cutting concerns for the `nlm` CLI: installation, authentication, global flags,
output formats, error codes, variable syntax, rate limits, environment variables,
security rules, and config file layout.

---

## 1. Installation

```bash
# Homebrew (macOS / Linux)
brew install newsleopard/tap/nlm

# Cargo (Rust toolchain required)
cargo install nlm-cli

# From source
git clone https://github.com/Newsleopard/nlm-open-cli.git && cd nlm-open-cli
cargo build --release
# binary at target/release/nlm
```

After installation, verify:

```bash
nlm --version
```

---

## 2. Authentication

### Interactive Setup

```bash
nlm config init
```

The wizard prompts for:

1. **EDM API Key** — obtain from the Newsleopard Dashboard (Settings > API Key).
2. **Surenotify API Key** — obtain from the Surenotify Dashboard (Settings > API Key).
3. **Default output format** — `json`, `table`, `yaml`, or `csv`.

Credentials are saved to `~/.config/nl/config.toml` with file permissions `0600`.

### Manual Setup

```bash
nlm config set edm_api_key "your-edm-key"
nlm config set sn_api_key  "your-sn-key"
```

### Environment Variable Overrides

Environment variables take highest precedence and override all config and flags:

```bash
export NL_EDM_API_KEY="your-edm-key"
export NL_SN_API_KEY="your-sn-key"
nlm edm account balance     # uses env var key
```

This is useful for CI/CD pipelines and Docker containers where config files are impractical.

---

## 3. Global Flags

Every `nlm` command accepts these flags:

| Flag | Short | Values | Default | Description |
|------|-------|--------|---------|-------------|
| `--format` | | `json`, `table`, `yaml`, `csv` | `json` | Output format for command results |
| `--profile` | | `NAME` | `default` | Config profile to use |
| `--dry-run` | | (flag) | off | Preview the HTTP request without executing it |
| `--verbose` | `-v` | `-v`, `-vv` | off | `-v` shows request summary; `-vv` shows full request/response details |
| `--quiet` | `-q` | (flag) | off | Suppress all output except errors |

Examples:

```bash
# Table output
nlm edm contacts list --format table

# Use a staging profile
nlm edm account balance --profile staging

# Preview what would be sent (no HTTP call made)
nlm edm campaign send 12345 --dry-run

# Debug a failing request
nlm sn email send --to user@example.com --subject "Hi" -vv
```

---

## 4. Output Formats

### JSON (default)

Pretty-printed when stdout is a TTY; **auto-switches to compact** when stdout is piped
or redirected. Paginated responses with `--page-all` use NDJSON (one JSON object per line).

```bash
nlm edm account balance
# {
#   "email": 10000,
#   "sms": 500
# }

# Piped — compact JSON:
nlm edm account balance | jq .email
```

### Table

Human-readable tables via `tabled`. Nested objects are auto-flattened.

```bash
nlm edm contacts list --format table
```

### YAML

```bash
nlm edm campaign get 12345 --format yaml
```

### CSV

```bash
nlm edm contacts list --format csv > contacts.csv
```

### Error Output

All errors are written to **stderr** as structured JSON, regardless of the chosen
`--format`. The envelope shape is:

```json
{
  "error": {
    "type": "Api",
    "message": "API error 403: [1001] Invalid API key",
    "exit_code": 1
  }
}
```

This allows scripts to parse stderr independently from stdout data.

---

## 5. Exit Codes

| Code | Meaning | NlError Variants | When |
|------|---------|------------------|------|
| **0** | Success | `DryRun`, `NoContent` | Command succeeded, or dry-run preview completed |
| **1** | API error | `Api` | HTTP 4xx/5xx response from the API |
| **2** | Validation | `Validation` | Invalid parameters, missing required fields |
| **3** | Auth / Config | `Auth`, `Config` | Invalid or missing API key, config file errors |
| **4** | Network / Rate limit | `Network`, `RateLimit` | Connection failure, DNS error, HTTP 429 after retries exhausted |
| **5** | IO | `Io` | File read/write failure, permission denied |

Usage in scripts:

```bash
nlm edm campaign send 12345
case $? in
  0) echo "Sent" ;;
  1) echo "API rejected the request" ;;
  2) echo "Bad parameters" ;;
  3) echo "Check your API key" ;;
  4) echo "Network issue or rate limited" ;;
  5) echo "File I/O problem" ;;
esac
```

---

## 6. Variable Syntax

The EDM and Surenotify APIs use **different** variable placeholder syntaxes.
Mixing them is a common mistake — `nlm` validates and warns on cross-use.

| API | Syntax | Example |
|-----|--------|---------|
| **EDM** (Newsleopard) | `${FIELD_NAME}` | `Hello ${NAME}, your code is ${CODE}` |
| **Surenotify** | `{{variable_name}}` | `Hello {{name}}, your code is {{code}}` |

**Rules:**

- EDM content containing `{{...}}` triggers a warning — likely meant `${...}`.
- Surenotify content containing `${...}` triggers a warning — likely meant `{{...}}`.
- The CLI does **not** block the request on a syntax mismatch; it warns on stderr so
  the user can correct if needed.

```bash
# Correct — EDM variable syntax:
nlm edm campaign update 123 --subject "Hello \${NAME}"

# Warning — wrong syntax for EDM:
nlm edm campaign update 123 --subject "Hello {{NAME}}"
# stderr: WARN: Found Surenotify-style {{NAME}} in EDM content. Did you mean ${NAME}?
```

---

## 7. Rate Limits

| Scope | Limit | Applies to |
|-------|-------|------------|
| General EDM | 2 requests/second | All `nlm edm` commands except report export |
| Report export | 1 request/10 seconds | `nlm edm report export` |
| Surenotify | No client-side limit | All `nlm sn` commands |

Rate limiting is enforced client-side using a token-bucket algorithm (governor crate).
When the bucket is empty, the CLI waits (does not fail) until a token is available.

### Retry Behavior

Server-side 429 and 5xx responses trigger automatic retries with exponential backoff:

- **Initial delay:** 500 ms
- **Max delay:** 30 s
- **Total timeout:** 120 s

If retries are exhausted, the CLI exits with code **4** (`RateLimit` or `Network`).
Non-transient errors (4xx other than 429) are permanent failures with no retry.

---

## 8. Environment Variables

| Variable | Description | Equivalent |
|----------|-------------|------------|
| `NL_EDM_API_KEY` | EDM API key | `nlm config set edm_api_key` |
| `NL_SN_API_KEY` | Surenotify API key | `nlm config set sn_api_key` |
| `NL_FORMAT` | Default output format | `--format` flag |
| `NL_PROFILE` | Active config profile name | `--profile` flag |
| `NL_MCP_URL` | MCP server base URL for tool discovery (`nlm` uses the `/mcp` endpoint) | `nlm config set mcp_url` |

### Precedence Order (highest to lowest)

1. **Environment variable** (`NL_*`)
2. **CLI flag** (`--format`, `--profile`)
3. **Profile config** (`[profiles.staging]` section)
4. **Default config** (`[default]` section)
5. **Built-in defaults** (format=json, profile=default)

---

## 9. Security Rules

- **Config file permissions:** `~/.config/nl/config.toml` is created with mode `0600`
  (owner read/write only). The CLI warns if permissions are looser.
- **API keys never in stdout:** Keys are masked in all output. `nlm config list` and
  `nlm config get edm_api_key` display `****...` instead of the actual value.
- **No key logging:** Even at `-vv` verbosity, the `x-api-key` header value is replaced
  with `****` in trace output.
- **Dry-run for destructive operations:** Use `--dry-run` before `send`, `delete`, or
  `update` commands to preview the exact HTTP request without executing it.
- **Environment variable hygiene:** Prefer `NL_EDM_API_KEY` / `NL_SN_API_KEY` in CI
  pipelines over storing keys in files; ensure they are not echoed in logs.

---

## 10. Config File

**Location:** `~/.config/nl/config.toml`

### Structure

```toml
[default]
edm_api_key = "your-edm-api-key"
sn_api_key = "your-sn-api-key"
default_format = "json"

[profiles.staging]
edm_api_key = "staging-edm-key"
sn_api_key = "staging-sn-key"
default_format = "table"

[profiles.production]
edm_api_key = "prod-edm-key"
sn_api_key = "prod-sn-key"
```

### Profile Management

```bash
# Create a new profile
nlm config profile create staging

# Set values within a profile
nlm config set edm_api_key "key" --profile staging
nlm config set sn_api_key "key" --profile staging

# Use a profile for a single command
nlm edm account balance --profile staging

# List all profiles
nlm config profile list

# Delete a profile
nlm config profile delete staging
```

### Config Commands

```bash
nlm config init          # Interactive setup wizard
nlm config set KEY VALUE # Set a config value (in default or --profile)
nlm config get KEY       # Get a config value (masked for API keys)
nlm config list          # Show all config (keys masked)
```
"#;
