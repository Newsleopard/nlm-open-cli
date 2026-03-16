# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`nl` is a Rust CLI tool wrapping the NewsLeopard EDM API (20 endpoints) and SureNotify API (11+ endpoints) for email marketing and transactional messaging. Binary name: `nl`, crate name: `nl-cli`.

**Status:** Pre-implementation — architecture docs and PRD are finalized in `docs/`, source code is to be built.

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

Layered architecture with static clap derive structs (not dynamic discovery — API surface is fixed at 31 endpoints):

```
main.rs → cli/ (clap derive) → executor/ → client/ + formatter/ + helpers/
                                               ↓
                                          共用層: config/, types/, error.rs
```

### Key Layers

- **`cli/`** — Command tree via clap derive. Two subtrees: `edm/` (contacts, campaign, ab_test, report, template, automation, account) and `sn/` (email, sms, webhook, domain)
- **`executor/`** — Routes parsed CLI args → client calls → formatted output
- **`client/`** — `ApiClient` (shared HTTP + rate limiter + dry-run), `EdmClient` (20 methods), `SureNotifyClient` (11+ methods), `rate_limiter.rs` (governor token bucket), `retry.rs` (backoff)
- **`formatter/`** — 4 output formats: JSON (pretty/compact/NDJSON), Table (tabled + auto-flatten), YAML, CSV
- **`helpers/`** — Orchestration workflows: `campaign_send`, `import_wait`, `report_export`, `domain_setup`
- **`config/`** — TOML config at `~/.config/nl/config.toml`, multi-profile, env var overrides
- **`types/`** — `edm.rs` and `surenotify.rs` request/response structs with serde rename attributes
- **`error.rs`** — `NlError` enum with 6 exit codes (0-5), JSON stderr output

### Two API Surfaces

| API | Base URL | Auth | Rate Limit | Variable Syntax |
|-----|----------|------|------------|-----------------|
| EDM | `api.newsleopard.com` | `x-api-key` header | 2 req/s general, 1 req/10s report export | `${FIELD_NAME}` |
| SureNotify | `mail.surenotifyapi.com` | `x-api-key` header | — | `{{variable_name}}` |

Variable syntax validation must warn when the wrong syntax is used (EDM content with `{{...}}` or SN content with `${...}`).

### Error System

`NlError` variants map to exit codes: `DryRun`/`NoContent` → 0, `Api` → 1, `Validation` → 2, `Auth`/`Config` → 3, `Network`/`RateLimit` → 4, `Io` → 5. All errors output JSON to stderr.

### Retry Logic

Exponential backoff (500ms initial, 30s max, 120s total timeout) on HTTP 429, 5xx, and network errors via the `backoff` crate. Non-transient errors are permanent failures.

## Testing Strategy

| Layer | Tool | What it covers |
|-------|------|----------------|
| Unit | `#[cfg(test)]` | Serialization, config parsing, variable validation |
| HTTP Mock | `wiremock` | All 31 endpoint request/response, error codes, rate limits |
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

- `docs/PRD.md` — Full API coverage, use cases, acceptance criteria, error code table
- `docs/Architecture.md` — Module structure, design patterns, type system, implementation phases
- `docs/CLI-USER-GUIDE.md` — Complete command tree with examples, all parameters documented

## Gotchas

- EDM and SureNotify use **different variable syntaxes** — must validate and warn on cross-use
- Config file permissions must be 600; API keys must never appear in stdout or logs
- `nl config list` must mask API key values (`****...`)
- JSON output auto-switches to compact when stdout is piped; `--page-all` uses NDJSON
- Report export has a separate, stricter rate limiter (1 req/10s) from general EDM calls (2 req/s)
- Helper commands (`nl helper` / `nl x`) orchestrate multiple API calls — they need both EDM and SN clients
- Eval alignment: CLI behavior should match `nlm-open-skills/evals/evals.json` test cases
