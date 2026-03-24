---
name: recipe-sms-notification
version: 1.0.0
description: "Send SMS notifications: configure webhooks and send messages with delivery tracking."
metadata:
  openclaw:
    category: "recipe"
    domain: "sms"
    requires:
      bins: ["nlm"]
      skills: ["nlm-sn-sms", "nlm-sn-sms-webhook"]
---

# Recipe: SMS Notification

Send SMS notifications with delivery tracking — configure a webhook for
delivery receipts, send a message, and verify delivery.

## Prerequisites

- Surenotify API key configured (`nlm config set sn_api_key "..."`)
- SMS credits in your Surenotify account
- A webhook endpoint URL for delivery receipts

---

## Steps

### Step 1 — Create a delivery webhook

Register a webhook to receive SMS delivery status updates.

```bash
nlm sn sms-webhook create \
  --event-type delivered \
  --url https://hooks.example.com/sms
```

### Step 2 — Send an SMS message

```bash
nlm sn sms send \
  --content 'ACME: Your order is shipped. Track at https://acme.com/track/123' \
  --phone 0912345678 \
  --country-code 886
```

The command outputs a message ID for tracking.

### Step 3 — Check delivery events

```bash
nlm sn sms events
```

Look for the message ID from Step 2 and confirm the status is `delivered`.

---

## Tips

- **Character limits:** Standard SMS messages are limited to 160 characters
  (70 for messages containing non-ASCII characters like Chinese). Messages
  exceeding this limit are split into multiple segments and billed accordingly.
- **Country codes:** Use the numeric country code without the `+` prefix.
  Taiwan is `886`, US is `1`, Japan is `81`.
- **URL shortening:** Long URLs consume valuable character space. Consider
  using a URL shortener to keep messages concise.
- **Opt-out compliance:** Include opt-out instructions in marketing SMS
  (e.g., "Reply STOP to unsubscribe"). Transactional messages (order
  confirmations, OTPs) are typically exempt.
