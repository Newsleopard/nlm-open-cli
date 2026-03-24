---
name: nlm-edm-ab-test
version: 0.1.2
description: "EDM A/B Test: Submit A/B test campaigns comparing subject lines, senders, or content."
metadata:
  openclaw:
    category: "group"
    domain: "ab-test"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM A/B Test

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Submit A/B test campaigns to optimize subject lines, sender identity, or content.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm ab-test submit` | Submit an A/B test campaign |
| `nlm edm ab-test submit-once` | One-time A/B test from a contacts file |

## Parameter Reference

### submit / submit-once

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--name` | Yes | Campaign name |
| `--lists` | Yes | Comma-separated list SNs to send to |
| `--test-on` | Yes | What to test: `subject`, `sender`, or `content` |
| `--proportion` | Yes | Percentage of recipients for test phase (e.g. `20`) |
| `--test-duration` | Yes | Duration of the test phase |
| `--test-unit` | Yes | Unit for duration: `hours` or `days` |

**Subject test fields** (when `--test-on subject`):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--subject-a` | Yes | Subject line for variant A |
| `--subject-b` | Yes | Subject line for variant B |
| `--from-name` | Yes | Sender display name |
| `--from-address` | Yes | Sender email address |
| `--html` / `--html-file` | Yes | Shared HTML content |

**Sender test fields** (when `--test-on sender`):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--from-name-a` | Yes | Sender name for variant A |
| `--from-address-a` | Yes | Sender address for variant A |
| `--from-name-b` | Yes | Sender name for variant B |
| `--from-address-b` | Yes | Sender address for variant B |
| `--subject` | Yes | Shared subject line |
| `--html` / `--html-file` | Yes | Shared HTML content |

**Content test fields** (when `--test-on content`):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--html-content-a-file` | Yes | HTML file for variant A |
| `--html-content-b-file` | Yes | HTML file for variant B |
| `--subject` | Yes | Shared subject line |
| `--from-name` | Yes | Sender display name |
| `--from-address` | Yes | Sender email address |

**Common optional fields** (all test types):

| Parameter | Required | Description |
|-----------|----------|-------------|
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

**submit-once only:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--contacts-file` | Yes | CSV/Excel file containing recipient contacts |

## Examples

```bash
# Test two subject lines on 20% of list, pick winner after 4 hours
nlm edm ab-test submit --name 'Subject Test' --lists L1 \
  --test-on subject --subject-a 'Free Shipping' --subject-b '50% Off' \
  --from-name ACME --from-address news@acme.com --html-file email.html \
  --proportion 20 --test-duration 4 --test-unit hours

# Test two content versions from a one-time contacts file
nlm edm ab-test submit-once --contacts-file contacts.csv \
  --name 'Content Test' --test-on content \
  --html-content-a-file version_a.html --html-content-b-file version_b.html \
  --subject 'Newsletter' --from-name ACME --from-address news@acme.com \
  --proportion 30 --test-duration 1 --test-unit days

# Dry-run to preview
nlm edm ab-test submit --name 'Sender Test' --lists L1 \
  --test-on sender \
  --from-name-a 'Sales Team' --from-address-a sales@acme.com \
  --from-name-b 'ACME News' --from-address-b news@acme.com \
  --subject 'Check this out' --html-file email.html \
  --proportion 25 --test-duration 6 --test-unit hours --dry-run
```

## Notes

- The test phase sends to `--proportion`% of recipients. After `--test-duration`,
  the winning variant is automatically sent to the remaining recipients.
- Winner is selected based on open rate by default.
- Required variant fields depend on the `--test-on` value — the CLI validates
  this at parse time.

