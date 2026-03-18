---
name: nlm-sn-email
version: 1.0.0
description: "Surenotify Email: Send transactional emails and query delivery events."
metadata:
  openclaw:
    category: "group"
    domain: "email"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-sn"]
---

# nlm sn email — Transactional Email

> **Prerequisites:** nlm-shared (global flags, auth, formats) and nlm-sn (Surenotify overview).

Send transactional emails with personalization variables and query delivery events.

---

## Commands

| Command | Description |
|---------|-------------|
| `nlm sn email send` | Send a transactional email (max 100 recipients per call) |
| `nlm sn email events` | Query email delivery events |

---

## nlm sn email send

Send a transactional email to one or more recipients.

### Required Flags

| Flag | Description |
|------|-------------|
| `--subject` | Email subject line |
| `--from-address` | Sender email address (must be from a verified domain) |
| `--html` | HTML body content |
| `--to` | Comma-separated recipient addresses |

### Optional Flags

| Flag | Description |
|------|-------------|
| `--from-name` | Sender display name |
| `--text` | Plain text body (fallback for non-HTML clients) |
| `--cc` | CC recipients (comma-separated) |
| `--bcc` | BCC recipients (comma-separated) |
| `--reply-to` | Reply-to address |
| `--variables` | JSON object of template variables using `{{var}}` syntax |
| `--recipients` | JSON array for per-recipient variables (replaces `--to`) |
| `--recipients-file` | Path to a JSON file with recipient data |
| `--html-file` | Path to an HTML file (replaces `--html`) |
| `--unsubscribe-link` | URL for the unsubscribe link |

### Examples

```bash
# Basic send
nlm sn email send \
  --subject "Order Confirmation" \
  --from-address "noreply@example.com" \
  --html "<h1>Your order is confirmed.</h1>" \
  --to alice@example.com

# With personalization variables
nlm sn email send \
  --subject "Welcome, {{name}}!" \
  --from-address "hello@example.com" \
  --from-name "Example Store" \
  --html "<h1>Hi {{name}}</h1><p>Your code is {{code}}.</p>" \
  --to alice@example.com \
  --variables '{"name":"Alice","code":"ABC123"}'

# Multiple recipients with per-recipient variables
nlm sn email send \
  --subject "Order {{order_id}} Shipped" \
  --from-address "noreply@example.com" \
  --html "<p>Hi {{name}}, order {{order_id}} is on its way.</p>" \
  --recipients '[
    {"name":"Alice","address":"alice@example.com","variables":{"name":"Alice","order_id":"ORD-001"}},
    {"name":"Bob","address":"bob@example.com","variables":{"name":"Bob","order_id":"ORD-002"}}
  ]'

# From file with CC and reply-to
nlm sn email send \
  --subject "Weekly Report" \
  --from-address "report@example.com" \
  --html-file weekly-report.html \
  --to team@example.com \
  --cc manager@example.com \
  --reply-to support@example.com

# Dry-run to preview the request
nlm sn email send \
  --subject "Test" \
  --from-address "noreply@example.com" \
  --html "<p>Test</p>" \
  --to test@example.com \
  --dry-run
```

---

## nlm sn email events

Query email delivery events by message ID or recipient address.

### Flags

| Flag | Description |
|------|-------------|
| `--id` | Message ID (mutually exclusive with `--recipient`) |
| `--recipient` | Recipient email address (mutually exclusive with `--id`) |
| `--start-date` | Start of date range (ISO 8601, e.g. `2026-03-01T00:00:00.00Z`) |
| `--end-date` | End of date range (ISO 8601) |
| `--email` | Filter by specific email address |
| `--event-type` | Filter by event type |
| `--status` | Filter by status: `accept`, `retry`, `delivery`, `open`, `click`, `bounce`, `complaint` |

### Examples

```bash
# Query by message ID
nlm sn email events --id "msg-uuid-123"

# Query by recipient with date range
nlm sn email events \
  --recipient alice@example.com \
  --start-date "2026-03-01T00:00:00.00Z" \
  --end-date "2026-03-15T23:59:59.00Z"

# Filter by status
nlm sn email events \
  --recipient alice@example.com \
  --status delivery

# Table output for readability
nlm sn email events --recipient alice@example.com --format table
```

---

## Notes

- **Variable syntax:** Use `{{variable_name}}` (double curly braces). Using EDM-style
  `${VARIABLE}` triggers a warning.
- **Recipient limit:** Maximum 100 recipients per `send` call.
- **`--id` and `--recipient` are mutually exclusive** — use one or the other for events.
- **Event history:** Up to 30 days, maximum 50 records per query.
- **Status values:** `accept`, `retry`, `delivery`, `open`, `click`, `bounce`, `complaint`.

