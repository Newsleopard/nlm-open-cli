---
name: nlm-edm-campaign
version: 0.1.2
description: "EDM Campaign: Submit, schedule, pause, delete, and analyze email campaigns."
metadata:
  openclaw:
    category: "group"
    domain: "campaign"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM Campaign

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Create, send, manage, and analyze email campaigns.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm campaign submit` | Submit a campaign for sending |
| `nlm edm campaign submit-once` | One-time campaign from a contacts file |
| `nlm edm campaign delete` | Delete one or more campaigns |
| `nlm edm campaign pause` | Pause a sending campaign |
| `nlm edm campaign status` | Check campaign sending status |
| `nlm edm campaign analyze` | AI-powered performance analysis (MCP) |
| `nlm edm campaign compare` | Compare 2-5 campaigns side by side (MCP) |
| `nlm edm campaign preflight` | Pre-send validation check (MCP) |
| `nlm edm campaign find` | Search campaigns by keyword (MCP) |
| `nlm edm campaign best-time` | Best send time recommendation (MCP) |

## Parameter Reference

### submit

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--name` | Yes | Campaign name |
| `--lists` | Yes | Comma-separated list SNs to send to |
| `--subject` | Yes | Email subject line |
| `--from-name` | Yes | Sender display name |
| `--from-address` | Yes | Sender email address |
| `--html` | No* | HTML content as inline string |
| `--html-file` | No* | Path to an HTML file |
| `--footer-lang` | No | Footer language: `chinese`, `english`, `japanese` (default: `chinese`) |
| `--preheader` | No | Preheader text |
| `--exclude-lists` | No | Comma-separated list SNs to exclude |
| `--schedule` | No | `immediate` (default) or `scheduled` |
| `--schedule-date` | No | Schedule date (e.g. `2025-03-20T09:00:00`) |
| `--schedule-timezone` | No | Timezone offset (e.g. `8` for UTC+8) |
| `--ga` | No | Enable Google Analytics tracking |
| `--ga-ecommerce` | No | Enable GA e-commerce tracking |
| `--utm-campaign` | No | Custom utm_campaign value |
| `--utm-content` | No | Custom utm_content value |

*One of `--html` or `--html-file` is required (mutually exclusive).

### submit-once

Same parameters as `submit` except `--lists` and `--exclude-lists` are
replaced by:

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--contacts-file` | Yes | CSV/Excel file containing recipient contacts |

### delete

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sns` | Yes | Comma-separated campaign SNs to delete |

### pause

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### status

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### analyze (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### compare (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sns` | Yes | 2-5 campaign SNs to compare |

### preflight (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### find (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `<query>` | Yes | Search query (positional argument) |

### best-time (MCP)

No parameters required.

## Examples

```bash
# Send immediately to list L1
nlm edm campaign submit --name 'March Newsletter' --lists L1 \
  --subject 'March Updates' --from-name 'ACME' \
  --from-address news@acme.com --html-file newsletter.html

# Schedule for a specific time
nlm edm campaign submit --name 'Promo' --lists L1,L2 --exclude-lists L3 \
  --subject 'Sale!' --from-name 'Shop' --from-address shop@acme.com \
  --html '<h1>50% Off</h1>' --schedule scheduled \
  --schedule-date '2025-03-20T09:00:00' --schedule-timezone 8

# Dry-run to preview the request
nlm edm campaign submit --name Test --lists L1 --subject Hi \
  --from-name Me --from-address me@x.com --html '<p>hi</p>' --dry-run

# One-time campaign from file
nlm edm campaign submit-once --contacts-file contacts.csv \
  --name 'One-time Blast' --subject 'Flash Sale' \
  --from-name Shop --from-address shop@acme.com --html-file promo.html

# Delete campaigns
nlm edm campaign delete --sns CAM001,CAM002

# Pause a sending campaign
nlm edm campaign pause --sn CAM12345

# Check campaign status
nlm edm campaign status --sn CAM12345

# AI analysis
nlm edm campaign analyze --sn CAM12345

# Compare campaigns
nlm edm campaign compare --sns CAM001 CAM002 CAM003

# Pre-flight check
nlm edm campaign preflight --sn CAM12345

# Search campaigns
nlm edm campaign find "March newsletter"

# Best send time
nlm edm campaign best-time
```

## Notes

- EDM uses `${FIELD_NAME}` variable syntax in subject and content. The CLI
  will warn if it detects Surenotify `{{variable}}` syntax.
- MCP commands (`analyze`, `compare`, `preflight`, `find`, `best-time`)
  require an MCP connection configured via `NL_MCP_URL`.
- The `compare` command accepts 2 to 5 campaign SNs.

