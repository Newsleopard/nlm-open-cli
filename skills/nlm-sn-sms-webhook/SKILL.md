---
name: nlm-sn-sms-webhook
version: 1.0.0
description: "Surenotify SMS Webhooks: Create, list, and delete SMS delivery webhooks."
metadata:
  openclaw:
    category: "group"
    domain: "sms-webhook"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-sn"]
---

# nlm sn sms-webhook — SMS Webhooks

> **Prerequisites:** nlm-shared (global flags, auth, formats) and nlm-sn (Surenotify overview).

Create, list, and delete webhooks for SMS delivery events. SMS webhooks provide
real-time push notifications when SMS events occur.

---

## Commands

| Command | Description |
|---------|-------------|
| `nlm sn sms-webhook create --event-type T --url U` | Create or update an SMS webhook |
| `nlm sn sms-webhook list` | List all SMS webhooks |
| `nlm sn sms-webhook delete --event-type T` | Delete an SMS webhook |

---

## nlm sn sms-webhook create

Create or update a webhook for a specific SMS event type.

### Required Flags

| Flag | Description |
|------|-------------|
| `--event-type` | Event type: `delivery`, `bounce` |
| `--url` | Webhook endpoint URL (must be HTTPS) |

### Event Types

| Event | Triggered when |
|-------|---------------|
| `delivery` | SMS successfully delivered to the recipient |
| `bounce` | SMS delivery failed |

### Examples

```bash
# Set up delivery notification webhook
nlm sn sms-webhook create \
  --event-type delivery \
  --url "https://api.example.com/sms-webhooks/delivery"

# Set up bounce notification webhook
nlm sn sms-webhook create \
  --event-type bounce \
  --url "https://api.example.com/sms-webhooks/bounce"

# Dry-run to preview
nlm sn sms-webhook create \
  --event-type delivery \
  --url "https://api.example.com/sms-webhooks/delivery" \
  --dry-run
```

---

## nlm sn sms-webhook list

List all configured SMS webhooks.

```bash
nlm sn sms-webhook list
nlm sn sms-webhook list --format table
```

---

## nlm sn sms-webhook delete

Delete an SMS webhook by event type.

```bash
nlm sn sms-webhook delete --event-type delivery
nlm sn sms-webhook delete --event-type bounce
```

---

## Notes

- **SMS has fewer event types** than email — only `delivery` and `bounce` are available.
- **One webhook per event type:** Creating a webhook for an event type that already has
  one will **update** the existing URL.
- **HTTPS required:** Webhook URLs should use HTTPS for secure delivery.
- **Payload format:** Webhook payloads are delivered as JSON POST requests to your endpoint.

