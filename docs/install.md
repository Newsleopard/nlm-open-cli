# nlm — Installation Guide

## For Humans

Copy this to your AI Agent (Claude Code, Cursor, Windsurf, etc.):

```
Help me install nlm: https://raw.githubusercontent.com/Newsleopard/nlm-open-cli/main/docs/install.md
```

That's it. Your agent will handle the rest.

---

## For AI Agents

### Goal

Install `nlm`, a CLI tool that wraps the [Newsleopard](https://www.newsleopard.com/) EDM API (email marketing) and Surenotify API (transactional email/SMS) into 34 subcommands with structured output, built-in rate limiting, and dry-run safety.

After installation, the user can manage email campaigns, contacts, templates, reports, transactional messages, and sender domains from the command line.

**Two API surfaces:**

| API | Purpose | Auth Variable |
|-----|---------|---------------|
| EDM (`api.newsleopard.com`) | Email marketing — campaigns, contacts, templates, reports | `NL_EDM_API_KEY` |
| Surenotify (`mail.surenotifyapi.com`) | Transactional — email, SMS, webhooks, domain verification | `NL_SN_API_KEY` |

The user may need one or both, depending on their use case.

### Boundaries

- **DO NOT** run commands with `sudo`
- **DO NOT** store API keys in stdout, logs, or files other than `~/.config/nl/config.toml`
- **DO NOT** echo or print API key values in any output
- **DO NOT** install packages or tools not listed in this guide
- If something requires elevated permissions, **tell the user** and let them decide

### Step 1: Install the binary

**Preferred — npm (no Rust toolchain needed):**

```bash
npm install -g @newsleopard/nlm-cli
```

**Fallback — GitHub Releases (if npm is unavailable):**

Download the pre-built binary for the user's platform from https://github.com/Newsleopard/nlm-open-cli/releases

| Platform | Target |
|----------|--------|
| macOS (Apple Silicon) | `aarch64-apple-darwin` |
| macOS (Intel) | `x86_64-apple-darwin` |
| Linux (x86_64) | `x86_64-unknown-linux-gnu` |
| Linux (arm64) | `aarch64-unknown-linux-gnu` |
| Windows (x86_64) | `x86_64-pc-windows-msvc` |

After downloading, make the binary executable (`chmod +x nlm`) and move it to a directory in `$PATH`.

**Fallback — Cargo (if Rust toolchain is available):**

```bash
cargo install nlm-cli
```

**Verify installation:**

```bash
nlm --version
```

Expected output: `nlm <version>` (e.g., `nlm 0.1.1`). If this fails, check that `nlm` is in `$PATH`.

### Step 2: Configure authentication

**For AI agents and CI/CD (non-interactive):**

Ask the user which API(s) they need, then set environment variables:

```bash
# EDM API (email marketing)
export NL_EDM_API_KEY="<ask user for key>"

# Surenotify API (transactional email/SMS)
export NL_SN_API_KEY="<ask user for key>"
```

**For persistent configuration:**

```bash
# Non-interactive — set keys directly
nlm config set edm_api_key "<key>"
nlm config set sn_api_key "<key>"

# Or interactive setup (only if user is present at terminal)
nlm config init
```

Config is stored at `~/.config/nl/config.toml` with file permissions 600.

**Credential precedence:** Environment variable > CLI flag > Profile config > `[default]` section.

### Step 3: Verify

Run a dry-run command to confirm the CLI works without making real API calls:

```bash
nlm edm contacts list-groups --dry-run
```

Expected: exit code 0, a JSON preview of the HTTP request that would be sent.

If the user provided an EDM API key, also verify with a real call:

```bash
nlm edm account balance
```

Expected: exit code 0, JSON with account balance information.

### Step 4: Report to user

Tell the user:
- Which install method was used
- Whether the API key is configured
- The result of the verification step
- A few example commands to get started:

```bash
# List contact groups
nlm edm contacts list-groups --format table

# Check campaign reports
nlm edm report list

# Send a transactional email (Surenotify)
nlm sn email send --to "user@example.com" --subject "Hello" --body "<p>Hi</p>"

# Preview any command without executing
nlm edm campaign submit --name "Newsletter" --dry-run
```

### Troubleshooting

| Problem | Solution |
|---------|----------|
| `npm: command not found` | Install Node.js from https://nodejs.org/ or use the GitHub Releases method |
| `cargo: command not found` | Install Rust from https://rustup.rs/ or use the npm/GitHub Releases method |
| `nlm: command not found` after npm install | Check `npm bin -g` is in `$PATH` |
| Exit code 3 (auth error) | API key is missing or invalid — re-run `nlm config set` or check env vars |
| Exit code 4 (network/rate limit) | Check internet connectivity; if rate limited, wait and retry |
| Permission denied on config file | Config must be at `~/.config/nl/config.toml` with `chmod 600` |

### Exit code reference

| Code | Meaning |
|------|---------|
| 0 | Success (also for dry-run and 204 No Content) |
| 1 | API error (HTTP 4xx/5xx) |
| 2 | Validation error (bad CLI arguments) |
| 3 | Auth/config error (invalid key, missing config) |
| 4 | Network/rate limit error |
| 5 | I/O error (file read/write failure) |

All errors output JSON to **stderr** with a `type` field.

---

## Quick Reference

| Command | What it does |
|---------|-------------|
| `nlm --version` | Show version |
| `nlm config init` | Interactive API key setup |
| `nlm config set edm_api_key "..."` | Set EDM key non-interactively |
| `nlm config set sn_api_key "..."` | Set Surenotify key non-interactively |
| `nlm config list` | Show current config (keys masked) |
| `nlm edm account balance` | Check account balance |
| `nlm edm contacts list-groups` | List contact groups |
| `nlm edm campaign submit --dry-run` | Preview campaign submission |
| `nlm edm report list` | List campaign reports |
| `nlm sn email send` | Send transactional email |
| `nlm sn sms send` | Send SMS |
| `nlm helper campaign-send` | Orchestrated campaign workflow |

**Global flags:** `--format <json|table|yaml|csv>`, `--profile <name>`, `--dry-run`, `--page-all`, `-v`/`-vv`, `-q`

**Full documentation:** https://github.com/Newsleopard/nlm-open-cli
