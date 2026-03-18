---
name: nlm-sn
version: 1.0.0
description: "Surenotify API: Transactional email and SMS — send messages, manage webhooks, and verify sender domains (14 endpoints)."
metadata:
  openclaw:
    category: "service"
    requires:
      bins: ["nlm"]
---

# nlm sn — Surenotify API Overview

The Surenotify API provides **transactional email and SMS** capabilities through
14 endpoints. Use it for order confirmations, verification codes, delivery
notifications, and any message triggered by user actions.

---

## API Summary

| Property | Value |
|----------|-------|
| **Base URL** | `mail.surenotifyapi.com` |
| **Authentication** | `x-api-key` header |
| **Variable syntax** | `{{variable_name}}` |
| **Rate limiting** | No client-side limit (server-enforced) |

---

## API Groups (14 endpoints)

| Group | Commands | Description |
|-------|----------|-------------|
| **Email** (`nlm sn email`) | `send`, `events` | Send transactional emails with personalization; query delivery events |
| **SMS** (`nlm sn sms`) | `send`, `events`, `exclusive-number` | Send SMS messages; query delivery events; list dedicated numbers |
| **Webhook** (`nlm sn webhook`) | `create`, `list`, `delete` | Manage email delivery webhooks |
| **SMS Webhook** (`nlm sn sms-webhook`) | `create`, `list`, `delete` | Manage SMS delivery webhooks |
| **Domain** (`nlm sn domain`) | `create`, `verify`, `remove` | Register and verify sender domains |

---

## Variable Syntax

Surenotify uses **double curly braces**: `{{variable_name}}`

```text
Hello {{name}}, your verification code is {{code}}.
```

> **Warning:** Using EDM-style `${VARIABLE}` in Surenotify content triggers a
> warning. The CLI does not block the request but alerts you on stderr so you
> can correct the syntax.

---

## Quick Start

```bash
# 1. Configure your Surenotify API key
nlm config set sn_api_key "your-sn-api-key"

# 2. Send a transactional email
nlm sn email send \
  --subject "Order Confirmation" \
  --from-address "noreply@example.com" \
  --html "<h1>Hi {{name}}</h1><p>Your order {{order_id}} is confirmed.</p>" \
  --to alice@example.com

# 3. Check delivery status
nlm sn email events --recipient alice@example.com

# 4. Set up a webhook for bounce notifications
nlm sn webhook create --event-type bounce --url "https://api.example.com/webhooks/bounce"
```

---

## Authentication

Requires a Surenotify API key. Set it via:

```bash
nlm config set sn_api_key "your-key"
# or
export NL_SN_API_KEY="your-key"
```

See **nlm-shared** for full authentication details, global flags, output formats,
and error codes.

