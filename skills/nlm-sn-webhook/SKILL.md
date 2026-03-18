---
name: nlm-sn-webhook
version: 1.0.0
description: "Surenotify Webhooks: Create, list, and delete email delivery webhooks."
metadata:
  openclaw:
    category: "group"
    domain: "webhook"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-sn"]
---

# nlm sn webhook — Email Webhooks

> **Prerequisites:** nlm-shared (global flags, auth, formats) and nlm-sn (Surenotify overview).

Create, list, and delete webhooks for email delivery events. Webhooks provide
real-time push notifications when email events occur.

---

## Commands

| Command | Description |
|---------|-------------|
| `nlm sn webhook create --event-type T --url U` | Create or update an email webhook |
| `nlm sn webhook list` | List all email webhooks |
| `nlm sn webhook delete --event-type T` | Delete an email webhook |

---

## nlm sn webhook create

Create or update a webhook for a specific email event type.

### Required Flags

| Flag | Description |
|------|-------------|
| `--event-type` | Event type: `delivered`, `opened`, `clicked`, `bounced`, `complained`, `unsubscribed` |
| `--url` | Webhook endpoint URL (must be HTTPS) |

### Event Types

| Event | Triggered when |
|-------|---------------|
| `delivered` | Email successfully delivered to recipient's mail server |
| `opened` | Recipient opened the email |
| `clicked` | Recipient clicked a link in the email |
| `bounced` | Email bounced (hard or soft bounce) |
| `complained` | Recipient marked the email as spam |
| `unsubscribed` | Recipient unsubscribed |

### Examples

```bash
# Set up bounce notification webhook
nlm sn webhook create \
  --event-type bounced \
  --url "https://api.example.com/webhooks/bounce"

# Set up open tracking webhook
nlm sn webhook create \
  --event-type opened \
  --url "https://api.example.com/webhooks/open"

# Set up all common webhooks
nlm sn webhook create --event-type delivered --url "https://api.example.com/webhooks/delivered"
nlm sn webhook create --event-type opened --url "https://api.example.com/webhooks/opened"
nlm sn webhook create --event-type clicked --url "https://api.example.com/webhooks/clicked"
nlm sn webhook create --event-type bounced --url "https://api.example.com/webhooks/bounced"
nlm sn webhook create --event-type complained --url "https://api.example.com/webhooks/complained"

# Dry-run to preview
nlm sn webhook create \
  --event-type bounced \
  --url "https://api.example.com/webhooks/bounce" \
  --dry-run
```

---

## nlm sn webhook list

List all configured email webhooks.

```bash
nlm sn webhook list
nlm sn webhook list --format table
```

---

## nlm sn webhook delete

Delete an email webhook by event type.

```bash
nlm sn webhook delete --event-type bounced
nlm sn webhook delete --event-type opened
```

---

## Notes

- **One webhook per event type:** Creating a webhook for an event type that already has
  one will **update** the existing URL.
- **HTTPS required:** Webhook URLs should use HTTPS for secure delivery.
- **Payload format:** Webhook payloads are delivered as JSON POST requests to your endpoint.
- Use `--dry-run` before creating webhooks to verify your configuration.

