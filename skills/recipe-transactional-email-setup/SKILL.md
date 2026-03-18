---
name: recipe-transactional-email-setup
version: 1.0.0
description: "Set up transactional email: verify domain, configure webhooks, and send a test email."
metadata:
  openclaw:
    category: "recipe"
    domain: "email"
    requires:
      bins: ["nlm"]
      skills: ["nlm-sn-email", "nlm-sn-domain", "nlm-sn-webhook"]
---

# Recipe: Transactional Email Setup

Set up transactional email from scratch — verify a sender domain, configure
delivery webhooks, send a test email, and confirm delivery.

## Prerequisites

- Surenotify API key configured (`nlm config set sn_api_key "..."`)
- Access to DNS management for your sending domain
- A webhook endpoint URL (e.g., `https://hooks.example.com/delivered`)

---

## Steps

### Step 1 — Set up and verify the sender domain

The `domain-setup` helper creates the domain, displays required DNS records,
and optionally waits for DNS propagation before verifying.

```bash
nlm helper domain-setup \
  --domain mail.example.com \
  --auto-verify-after 60
```

The `--auto-verify-after 60` flag waits 60 seconds then attempts verification.
Add the displayed CNAME and TXT records to your DNS provider while waiting.

### Step 2 — Create delivery webhooks

Register a webhook endpoint to receive real-time delivery events.

```bash
nlm sn webhook create \
  --event-type delivered \
  --url https://hooks.example.com/delivered
```

Repeat for other event types as needed: `bounced`, `opened`, `clicked`,
`complained`, `unsubscribed`.

### Step 3 — Send a test email

```bash
nlm sn email send \
  --subject 'Test' \
  --from-address noreply@mail.example.com \
  --html '<p>Test email</p>' \
  --to test@example.com
```

### Step 4 — Check delivery events

Verify the test email was delivered and the webhook fired.

```bash
nlm sn email events --email test@example.com
```

Look for a `delivered` event. If you see `bounced` instead, check your DNS
records and sender domain verification status.

---

## Tips

- **DNS propagation:** CNAME and TXT records can take up to 48 hours to
  propagate, though most providers update within minutes. Use
  `dig CNAME mail.example.com` to check locally.
- **Multiple event types:** Create separate webhooks for `delivered`,
  `bounced`, and `complained` to route events to different handlers.
- **Webhook security:** Use HTTPS endpoints and validate the webhook
  signature header to prevent spoofing.
- **Variables:** Use `{{variable_name}}` syntax in Surenotify templates.
  Remember: this is different from EDM's `${FIELD_NAME}` syntax.
