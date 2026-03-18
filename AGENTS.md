# nlm — AI Agent Guide

CLI for the Newsleopard email marketing (EDM) API and Surenotify transactional messaging API. Wraps 34 REST endpoints into a single binary with JSON output, exit codes, and dry-run support.

## Quick Setup

```bash
# Install
brew install newsleopard/tap/nlm   # or: cargo install nlm-cli

# Configure (interactive)
nlm config init

# Or via environment variables (no config file needed)
export NL_EDM_API_KEY="your-edm-key"
export NL_SN_API_KEY="your-sn-key"

# Verify
nlm edm account balance
```

## Command Tree

### EDM API — Bulk Marketing (20 endpoints)

| Command | Description |
| ------- | ----------- |
| `nlm edm contacts create-group --name NAME` | Create a contact group |
| `nlm edm contacts list-groups` | List all contact groups |
| `nlm edm contacts import-file --list-sn SN --file FILE` | Import contacts from CSV/Excel |
| `nlm edm contacts import-text --list-sn SN --csv-text TEXT` | Import contacts from inline CSV |
| `nlm edm contacts import-status --sn SN` | Check import job status |
| `nlm edm contacts remove --list-sn SN --field F --op OP --value V` | Remove contacts by filter |
| `nlm edm contacts top-lists` | Top-performing lists by engagement (MCP) |
| `nlm edm campaign submit --name N --lists L --subject S --from-name F --from-address A --html H` | Submit a campaign |
| `nlm edm campaign submit-once --contacts-file F --name N --subject S ...` | One-time campaign from file |
| `nlm edm campaign delete --sns SN1,SN2` | Delete campaigns |
| `nlm edm campaign pause --sn SN` | Pause a sending campaign |
| `nlm edm campaign status --sn SN` | Check campaign status |
| `nlm edm campaign analyze --sn SN` | AI-powered analysis (MCP) |
| `nlm edm campaign compare --sns SN1 SN2` | Compare campaigns side by side (MCP) |
| `nlm edm campaign preflight --sn SN` | Pre-send validation (MCP) |
| `nlm edm campaign find "query"` | Search campaigns (MCP) |
| `nlm edm campaign best-time` | Best send time recommendation (MCP) |
| `nlm edm ab-test submit ...` | Submit A/B test campaign |
| `nlm edm ab-test submit-once ...` | One-time A/B test from file |
| `nlm edm report list --start DATE --end DATE` | List reports by date range |
| `nlm edm report metrics --sns SN1,SN2` | Get campaign metrics |
| `nlm edm report export --sn SN` | Export report (async) |
| `nlm edm report download-link --sn SN` | Get export download link |
| `nlm edm report summary` | Recent campaigns summary (MCP) |
| `nlm edm report clicks --sn SN` | Per-link click breakdown (MCP) |
| `nlm edm template list` | List all templates |
| `nlm edm template get --id ID` | Get template by ID |
| `nlm edm template save --sn SN --name NAME` | Save campaign as template (MCP) |
| `nlm edm automation trigger --script-sn SN --event E --recipients R` | Trigger automation |
| `nlm edm account balance` | Check email/SMS credits |

### Surenotify API — Transactional Messaging (14 endpoints)

| Command | Description |
| ------- | ----------- |
| `nlm sn email send --subject S --from-address A --html H --to R` | Send transactional email |
| `nlm sn email events` | Query email delivery events |
| `nlm sn sms send --content C --phone P --country-code CC` | Send SMS |
| `nlm sn sms events` | Query SMS delivery events |
| `nlm sn sms exclusive-number` | List dedicated SMS numbers |
| `nlm sn webhook create --event-type T --url U` | Create email webhook |
| `nlm sn webhook list` | List email webhooks |
| `nlm sn webhook delete --event-type T` | Delete email webhook |
| `nlm sn sms-webhook create --event-type T --url U` | Create SMS webhook |
| `nlm sn sms-webhook list` | List SMS webhooks |
| `nlm sn sms-webhook delete --event-type T` | Delete SMS webhook |
| `nlm sn domain create --domain D` | Register sender domain |
| `nlm sn domain verify --domain D` | Verify domain DNS |
| `nlm sn domain remove --domain D` | Remove sender domain |

### MCP — AI Tool Discovery

| Command | Description |
| ------- | ----------- |
| `nlm mcp tools` | List all MCP tools with descriptions and JSON schemas |
| `nlm mcp call TOOL_NAME --json '{"key":"val"}'` | Invoke any MCP tool by name |

### Helper — Multi-Step Orchestration

| Command | Description |
| ------- | ----------- |
| `nlm helper campaign-send --name N --lists L --subject S ... --wait` | Submit + wait for completion |
| `nlm helper import-and-wait --list-sn SN --file F` | Import contacts + poll until done |
| `nlm helper report-download --sn SN --output FILE` | Export report + download file |
| `nlm helper domain-setup --domain D --auto-verify-after 60` | Create domain + verify DNS |

### Config

| Command | Description |
| ------- | ----------- |
| `nlm config init` | Interactive first-time setup |
| `nlm config set KEY VALUE` | Set a config value |
| `nlm config get KEY` | Get a config value |
| `nlm config list` | Show all settings (keys masked) |
| `nlm config profile create NAME` | Create a profile |
| `nlm config profile list` | List profiles |
| `nlm config profile delete NAME` | Delete a profile |

## Common Tasks

| Task | Command |
| ---- | ------- |
| Send a campaign | `nlm helper campaign-send --name 'Newsletter' --lists L1 --subject 'News' --from-name ACME --from-address news@acme.com --html-file content.html --wait` |
| Check account balance | `nlm edm account balance` |
| Import contacts and wait | `nlm helper import-and-wait --list-sn L1 --file contacts.csv` |
| Send transactional email | `nlm sn email send --subject 'Order Confirmed' --from-address noreply@acme.com --html '<p>Your order {{order_id}} is confirmed.</p>' --to customer@example.com` |
| Export campaign report | `nlm helper report-download --sn CAM12345 --output report.csv` |
| Discover MCP tools | `nlm mcp tools` |
| Preview without executing | Add `--dry-run` to any command |

## Global Flags

| Flag | Description | Default |
| ---- | ----------- | ------- |
| `--format json\|table\|yaml\|csv` | Output format | `json` |
| `--profile NAME` | Config profile | `default` |
| `--dry-run` | Preview HTTP request without executing | off |
| `-v` / `-vv` | Verbose output (summary / full) | off |
| `-q` | Quiet mode (errors only) | off |

## Output & Scripting

- **JSON by default** — auto-switches to compact when stdout is piped
- **Exit codes**: 0 = success, 1 = API error, 2 = validation, 3 = auth/config, 4 = network/rate-limit, 5 = IO
- **Errors**: JSON to stderr with `{"error": "...", "code": N}`
- **Piping**: `nlm edm report list --start 2025-01-01 --end 2025-01-31 | jq '.[] | .sn'`

## Environment Variables

| Variable | Description | Overrides |
| -------- | ----------- | --------- |
| `NL_EDM_API_KEY` | EDM API key | config file |
| `NL_SN_API_KEY` | Surenotify API key | config file |
| `NL_FORMAT` | Default output format | `--format` flag |
| `NL_PROFILE` | Active profile name | `--profile` flag |
| `NL_MCP_URL` | MCP server URL | config file |

**Precedence**: env var > CLI flag > profile setting > default

## AI Agent Skills

Run `nlm generate-skills` to generate 35 skill files in `skills/` that teach AI agents how to use every nlm command. Skills cover shared patterns, API groups, helper workflows, recipes, and role-based personas. See `docs/skills.md` for the full index.

## Key Constraints

- **Variable syntax**: EDM uses `${FIELD_NAME}`, Surenotify uses `{{variable_name}}` — do not mix
- **Rate limits**: 2 req/s for general EDM calls, 1 req/10s for report exports
- **Config security**: `~/.config/nl/config.toml` has 600 permissions; API keys never appear in stdout
- **SMS regulation**: SMS content must include company name (NCC requirement)
- **Recipients limit**: Surenotify max 100 recipients per request
