---
name: nlm-helper-campaign-send
version: 1.0.0
description: "nlm helper: Submit a campaign and optionally wait for completion — combines submit + status polling."
metadata:
  openclaw:
    category: "helper"
    domain: "campaign"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm", "nlm-edm-campaign"]
---

# nlm helper campaign-send

> **Prerequisite skills:** `nlm-shared`, `nlm-edm`, `nlm-edm-campaign`

Submit a campaign and optionally wait for completion. This helper combines
balance check, campaign submit, status polling, and final metrics retrieval
into a single command.

## Usage

```bash
nlm helper campaign-send \
  --name <NAME> --lists <LIST_SNs> --subject <SUBJECT> \
  --from-name <NAME> --from-address <EMAIL> \
  (--html <HTML> | --html-file <PATH>) \
  [--wait] [options...]
```

Alias: `nlm x campaign-send ...`

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `--name` | Yes | — | Campaign name |
| `--lists` | Yes | — | Comma-separated contact list SNs to send to |
| `--subject` | Yes | — | Email subject line (supports `${FIELD}` variables) |
| `--from-name` | Yes | — | Sender display name |
| `--from-address` | Yes | — | Sender email address |
| `--html` | One of html/html-file | — | Inline HTML content |
| `--html-file` | One of html/html-file | — | Path to an HTML file |
| `--footer-lang` | No | `chinese` | Footer language: `chinese`, `english`, `japanese` |
| `--preheader` | No | — | Preheader text (inbox preview) |
| `--exclude-lists` | No | — | Comma-separated list SNs to exclude |
| `--schedule` | No | `immediate` | `immediate` or `scheduled` |
| `--schedule-date` | No | — | Schedule date (e.g. `2025-01-15T09:00:00`) |
| `--schedule-timezone` | No | — | Timezone offset (e.g. `8` for UTC+8) |
| `--ga` | No | `false` | Enable Google Analytics tracking |
| `--ga-ecommerce` | No | `false` | Enable GA e-commerce tracking |
| `--utm-campaign` | No | — | Custom `utm_campaign` value |
| `--utm-content` | No | — | Custom `utm_content` value |
| `--wait` | No | `false` | Poll campaign status until sending completes |

## Workflow

1. **Check balance** — verifies sufficient email credits before sending.
2. **Submit campaign** — builds the request from parameters and submits it.
3. **(if `--wait`)** **Poll status** — spinner polls every 5 s (timeout 600 s).
4. **(if `--wait`)** **Return metrics** — once COMPLETE/SENT, fetches final
   performance metrics (opens, clicks, bounces).

## Examples

```bash
# Send immediately and wait for completion
nlm helper campaign-send \
  --name "March Newsletter" \
  --lists GRP-001,GRP-002 \
  --subject "March deals inside" \
  --from-name "ACME Corp" \
  --from-address news@acme.com \
  --html-file ./march-newsletter.html \
  --wait

# Schedule for later (no waiting needed)
nlm x campaign-send \
  --name "Holiday Sale" \
  --lists GRP-001 \
  --subject "Holiday specials for ${FIRST_NAME}" \
  --from-name "ACME" \
  --from-address promo@acme.com \
  --html-file sale.html \
  --schedule scheduled \
  --schedule-date "2025-12-20T09:00:00" \
  --schedule-timezone 8 \
  --ga --utm-campaign "holiday-2025"

# Dry-run preview (no API call)
nlm helper campaign-send \
  --name "Test" --lists GRP-001 --subject "Hi" \
  --from-name "Test" --from-address test@example.com \
  --html "<p>Hello</p>" --dry-run
```

## Tips

- Use `--dry-run` to preview the request payload without sending.
- The `--html` and `--html-file` flags are mutually exclusive.
- EDM variable syntax is `${FIELD_NAME}`. Using `{{...}}` triggers a warning.
- Without `--wait`, the command returns immediately after submit with the
  campaign SN. You can check status later with `nlm edm campaign status`.
- Balance check failure (zero credits) returns exit code 2 (Validation).

