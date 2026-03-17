# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`nlm` is a Rust CLI tool wrapping the Newsleopard EDM API (20 endpoints) and Surenotify API (14 endpoints) for email marketing and transactional messaging. Binary name: `nlm`, crate name: `nlm-cli`.

**Status:** Implemented — all modules complete and compiling. Internal design docs in `docs/internal/`.

## Build & Development Commands

```bash
cargo build                          # Build
cargo build --release                # Release build (thin LTO + strip)
cargo test                           # Run all tests
cargo test edm_api_test              # Run specific test module
cargo test cli_integration_test      # Run CLI integration tests
cargo clippy -- -D warnings          # Lint (treat warnings as errors)
cargo fmt                            # Format code
cargo fmt -- --check                 # Check formatting without modifying
```

**Rust version:** 1.75+ (edition 2021)

## Architecture

Layered architecture with static clap derive structs (not dynamic discovery — API surface is fixed at 34 endpoints):

```
main.rs → cli/ (clap derive) → executor/ → client/ + formatter/ + helpers/
                  │                            ↓
                  └─ config commands ──→  config/ (no API client needed)
                                         types/, error.rs
```

### Key Layers

- **`cli/`** — Command tree via clap derive. Four subtrees: `edm/` (contacts, campaign, ab_test, report, template, automation, account), `sn/` (email, sms, webhook, sms_webhook, domain), `mcp` (tools, call), `config` (init, set, get, list, profile). Also `helper`/`x` for orchestration workflows
- **`executor/`** — Routes parsed CLI args → client calls → formatted output
- **`client/`** — `ApiClient` (shared HTTP + rate limiter + dry-run), `EdmClient` (20 methods), `SurenotifyClient` (14 methods), `McpClient` (JSON-RPC 2.0 tool discovery/invocation via `mcp.rs`), `rate_limiter.rs` (governor token bucket), `retry.rs` (backoff)
- **`formatter/`** — 4 output formats: JSON (pretty/compact/NDJSON), Table (tabled + auto-flatten), YAML, CSV
- **`helpers/`** — Orchestration workflows: `campaign_send`, `import_wait`, `report_export`, `domain_setup`
- **`config/`** — TOML config at `~/.config/nl/config.toml`, multi-profile, env var overrides (`NL_EDM_API_KEY`, `NL_SN_API_KEY`, `NL_FORMAT`, `NL_MCP_URL`)
- **`types/`** — `edm.rs` and `surenotify.rs` request/response structs with serde rename attributes
- **`error.rs`** — `NlError` enum with 6 exit codes (0-5), JSON stderr output

### Two API Surfaces

| API | Base URL | Auth | Rate Limit | Variable Syntax |
|-----|----------|------|------------|-----------------|
| EDM | `api.newsleopard.com` | `x-api-key` header | 2 req/s general, 1 req/10s report export | `${FIELD_NAME}` |
| Surenotify | `mail.surenotifyapi.com` | `x-api-key` header | — | `{{variable_name}}` |

Variable syntax validation must warn when the wrong syntax is used (EDM content with `{{...}}` or SN content with `${...}`).

### Error System

`NlError` variants map to exit codes: `DryRun`/`NoContent` → 0, `Api` → 1, `Validation` → 2, `Auth`/`Config` → 3, `Network`/`RateLimit` → 4, `Io` → 5. All errors output JSON to stderr.

### Retry Logic

Exponential backoff (500ms initial, 30s max, 120s total timeout) on HTTP 429, 5xx, and network errors via the `backoff` crate. Non-transient errors are permanent failures.

## Testing Strategy

| Layer | Tool | What it covers |
|-------|------|----------------|
| Unit | `#[cfg(test)]` | Serialization, config parsing, variable validation |
| HTTP Mock | `wiremock` | MCP client (handshake, tool calls, errors, retry); EDM/SN tests partial |
| CLI E2E | `assert_cmd` | Command parsing, exit codes, dry-run, output formats |
| Snapshot | `insta` | Formatter output stability |

## Key Dependencies

- `clap` (derive) for CLI, `reqwest` (rustls, no OpenSSL) for HTTP, `tokio` for async
- `governor` for token bucket rate limiting, `backoff` for retry
- `tabled` for table output, `indicatif` for progress bars in helper workflows
- `thiserror` for error types, `tracing`/`tracing-subscriber` for structured logging

## CI/CD

GitHub Actions: `cargo fmt --check` → `cargo clippy` → `cargo test` on every push/PR. Release on `v*` tags cross-compiles for 5 targets: Linux x86_64/arm64, macOS x86_64/arm64, Windows x86_64.

## Reference Documents

- `docs/CLI-USER-GUIDE.md` — Complete command tree with examples, all parameters documented
- `docs/internal/PRD.md` — (Internal) Full API coverage, use cases, acceptance criteria, error code table
- `docs/internal/Architecture.md` — (Internal) Module structure, design patterns, type system, implementation phases

## Gotchas

- **`cargo` may not be in PATH** — use `$HOME/.cargo/bin/cargo` if `cargo` command is not found
- EDM and Surenotify use **different variable syntaxes** — must validate and warn on cross-use
- Config file permissions must be 600; API keys must never appear in stdout or logs
- `nlm config list` must mask API key values (`****...`)
- JSON output auto-switches to compact when stdout is piped; `--page-all` uses NDJSON
- Report export has a separate, stricter rate limiter (1 req/10s) from general EDM calls (2 req/s)
- Helper commands (`nlm helper` / `nlm x`) orchestrate multiple API calls — they need both EDM and SN clients
- Eval alignment: CLI behavior should match `nlm-open-skills/evals/evals.json` test cases
