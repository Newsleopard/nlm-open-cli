---
name: nlm-edm
version: 0.1.2
description: "Newsleopard EDM API: Bulk email marketing — campaigns, contacts, reports, templates, A/B tests, automations, and account (20 endpoints)."
metadata:
  openclaw:
    category: "service"
    requires:
      bins: ["nlm"]
---

# Newsleopard EDM API

The EDM (Email Direct Marketing) API provides bulk email marketing capabilities
through `api.newsleopard.com`.

## Authentication

All requests require an `x-api-key` header. Configure via:

```bash
nlm config set edm_api_key YOUR_KEY
```

Or set the `NL_EDM_API_KEY` environment variable.

## Rate Limits

- **General:** 2 requests/second (token bucket)
- **Report export:** 1 request/10 seconds (stricter limit)

The CLI enforces these limits automatically with a built-in rate limiter.

## Variable Syntax

EDM content uses `${FIELD_NAME}` for personalization variables (e.g.
`${EMAIL}`, `${NAME}`). Do **not** use the Surenotify `{{variable}}`
syntax in EDM content — the CLI will warn if it detects cross-syntax usage.

## Subcommand Groups

| Group | Description |
|-------|-------------|
| `nlm edm contacts` | Manage contact groups and imports |
| `nlm edm campaign` | Create, send, pause, and analyze campaigns |
| `nlm edm ab-test` | A/B test campaigns (subject, sender, or content) |
| `nlm edm report` | Campaign reports, metrics, and exports |
| `nlm edm template` | List, retrieve, and save templates |
| `nlm edm automation` | Trigger automation workflows |
| `nlm edm account` | Account balance and credits |

## Global Flags

All EDM commands support these flags (see `nlm-shared` skill for details):

- `--format json|table|yaml|csv` — output format
- `--dry-run` — preview the request without sending
- `--profile NAME` — use a named config profile
- `--verbose` — enable debug logging

## Examples

```bash
# List contact groups as a table
nlm edm contacts list-groups --format table

# Submit a campaign
nlm edm campaign submit --name 'March Newsletter' --lists L1 \
  --subject 'March Updates' --from-name 'ACME' \
  --from-address news@acme.com --html-file newsletter.html

# Check account balance
nlm edm account balance
```

