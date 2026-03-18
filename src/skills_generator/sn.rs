//! Surenotify skills: service overview + 5 API group skills.

use super::{SkillCategory, SkillDefinition};

pub fn skills() -> Vec<SkillDefinition> {
    vec![
        // 1. Service overview
        SkillDefinition {
            name: "nlm-sn".to_string(),
            version: "1.0.0".to_string(),
            description: "Surenotify API: Transactional email and SMS \u{2014} send messages, manage webhooks, and verify sender domains (14 endpoints).".to_string(),
            category: SkillCategory::Service,
            domain: None,
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![],
            body: NLM_SN.to_string(),
        },
        // 2. Email group
        SkillDefinition {
            name: "nlm-sn-email".to_string(),
            version: "1.0.0".to_string(),
            description: "Surenotify Email: Send transactional emails and query delivery events.".to_string(),
            category: SkillCategory::Group,
            domain: Some("email".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-sn".to_string()],
            body: NLM_SN_EMAIL.to_string(),
        },
        // 3. SMS group
        SkillDefinition {
            name: "nlm-sn-sms".to_string(),
            version: "1.0.0".to_string(),
            description: "Surenotify SMS: Send SMS messages and query delivery events.".to_string(),
            category: SkillCategory::Group,
            domain: Some("sms".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-sn".to_string()],
            body: NLM_SN_SMS.to_string(),
        },
        // 4. Email webhook group
        SkillDefinition {
            name: "nlm-sn-webhook".to_string(),
            version: "1.0.0".to_string(),
            description: "Surenotify Webhooks: Create, list, and delete email delivery webhooks.".to_string(),
            category: SkillCategory::Group,
            domain: Some("webhook".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-sn".to_string()],
            body: NLM_SN_WEBHOOK.to_string(),
        },
        // 5. SMS webhook group
        SkillDefinition {
            name: "nlm-sn-sms-webhook".to_string(),
            version: "1.0.0".to_string(),
            description: "Surenotify SMS Webhooks: Create, list, and delete SMS delivery webhooks.".to_string(),
            category: SkillCategory::Group,
            domain: Some("sms-webhook".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-sn".to_string()],
            body: NLM_SN_SMS_WEBHOOK.to_string(),
        },
        // 6. Domain group
        SkillDefinition {
            name: "nlm-sn-domain".to_string(),
            version: "1.0.0".to_string(),
            description: "Surenotify Domains: Register, verify, and remove sender domains.".to_string(),
            category: SkillCategory::Group,
            domain: Some("domain".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-sn".to_string()],
            body: NLM_SN_DOMAIN.to_string(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Body content constants
// ---------------------------------------------------------------------------

const NLM_SN: &str = r#"# nlm sn — Surenotify API Overview

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
"#;

const NLM_SN_EMAIL: &str = r#"# nlm sn email — Transactional Email

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
"#;

const NLM_SN_SMS: &str = r#"# nlm sn sms — SMS Messaging

> **Prerequisites:** nlm-shared (global flags, auth, formats) and nlm-sn (Surenotify overview).

Send SMS messages, query delivery events, and list dedicated phone numbers.

---

## Commands

| Command | Description |
|---------|-------------|
| `nlm sn sms send` | Send an SMS message |
| `nlm sn sms events` | Query SMS delivery events |
| `nlm sn sms exclusive-number` | List dedicated SMS numbers |

---

## nlm sn sms send

Send an SMS message to one or more recipients.

### Required Flags

| Flag | Description |
|------|-------------|
| `--content` | SMS body (must include company/brand name per NCC regulation) |
| `--phone` | Recipient phone number (digits only, no `+`, `-`, or spaces) |
| `--country-code` | Country calling code (e.g. `886` for Taiwan, `1` for US) |

### Optional Flags

| Flag | Description |
|------|-------------|
| `--from` | Sender phone number (use a dedicated exclusive number) |
| `--alive-mins` | Message validity period in minutes |
| `--recipients` | JSON array for batch sending with per-recipient variables |
| `--recipients-file` | Path to a JSON file with recipient data |

### Examples

```bash
# Basic SMS send
nlm sn sms send \
  --content "【Example Store】Your verification code is 123456" \
  --phone 0912345678 \
  --country-code 886

# With template variables
nlm sn sms send \
  --content "【Example Store】Hi {{name}}, your code is {{code}}" \
  --phone 0912345678 \
  --country-code 886

# Batch send with per-recipient variables
nlm sn sms send \
  --content "【Example Store】Dear {{name}}, order {{order_id}} has shipped" \
  --recipients '[
    {"address":"0912345678","country_code":"886","variables":{"name":"Alice","order_id":"A001"}},
    {"address":"0923456789","country_code":"886","variables":{"name":"Bob","order_id":"A002"}}
  ]'

# Using a dedicated number with retry window
nlm sn sms send \
  --content "【Brand】Verification code: {{code}}" \
  --phone 0912345678 \
  --country-code 886 \
  --from 0900123456 \
  --alive-mins 30

# Dry-run to preview
nlm sn sms send \
  --content "【Brand】Test message" \
  --phone 0912345678 \
  --country-code 886 \
  --dry-run
```

---

## nlm sn sms events

Query SMS delivery events by message ID or recipient.

### Flags

| Flag | Description |
|------|-------------|
| `--id` | Message ID (mutually exclusive with `--recipient`) |
| `--recipient` | Recipient phone number (mutually exclusive with `--id`) |
| `--country-code` | Country code (required when using `--recipient`) |
| `--from` | Start of date range (ISO 8601) |
| `--status` | Filter by status: `accept`, `delivery`, `bounce` |

### Examples

```bash
# Query by message ID
nlm sn sms events --id "msg-uuid-456"

# Query by recipient
nlm sn sms events --recipient 0912345678 --country-code 886

# With date filter and status
nlm sn sms events \
  --recipient 0912345678 \
  --country-code 886 \
  --from "2026-03-01T00:00:00.00Z" \
  --status delivery
```

---

## nlm sn sms exclusive-number

List all dedicated SMS phone numbers assigned to your account.

```bash
nlm sn sms exclusive-number
nlm sn sms exclusive-number --format table
```

Example output:

```json
{
  "phoneNumbers": [
    {
      "phoneNumber": "0900123456",
      "createDate": "2026-01-15T08:00:00Z",
      "updateDate": "2026-01-15T08:00:00Z"
    }
  ]
}
```

---

## Notes

- **NCC regulation:** SMS content **must** include the company or brand name (e.g. `【Brand Name】`).
- **Phone number format:** Digits only — no `+`, `-`, or spaces.
- **URLs in SMS:** URLs included in SMS content require a whitelist application.
- **Variable syntax:** Use `{{variable_name}}` (double curly braces).
- **SMS status values:** `accept`, `delivery`, `bounce`.
"#;

const NLM_SN_WEBHOOK: &str = r#"# nlm sn webhook — Email Webhooks

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
"#;

const NLM_SN_SMS_WEBHOOK: &str = r#"# nlm sn sms-webhook — SMS Webhooks

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
"#;

const NLM_SN_DOMAIN: &str = r#"# nlm sn domain — Sender Domain Verification

> **Prerequisites:** nlm-shared (global flags, auth, formats) and nlm-sn (Surenotify overview).

Register, verify, and remove sender domains for the Surenotify API. Domain
verification ensures your emails pass SPF and DKIM authentication.

---

## Commands

| Command | Description |
|---------|-------------|
| `nlm sn domain create --domain D` | Register a sender domain (returns DNS records to configure) |
| `nlm sn domain verify --domain D` | Verify domain DNS configuration |
| `nlm sn domain remove --domain D` | Remove a sender domain |

---

## nlm sn domain create

Register a new sender domain. The API returns DNS records (SPF and DKIM) that
must be added to your domain's DNS configuration.

### Required Flags

| Flag | Description |
|------|-------------|
| `--domain` | Domain name to register (e.g. `mail.example.com`) |

### Examples

```bash
nlm sn domain create --domain mail.example.com
```

Example output:

```json
[
  {
    "name": "mail.example.com",
    "value": "v=spf1 include:amazonses.com include:mailgun.org ?all",
    "record_type": 0,
    "valid": false
  },
  {
    "name": "selector._domainkey.mail.example.com",
    "value": "...",
    "record_type": 1,
    "valid": false
  }
]
```

**Record types:** `0` = TXT record, `1` = CNAME record.

---

## nlm sn domain verify

Check whether the required DNS records have been correctly configured. Returns
the same DNS record array as `create`, with updated `valid` fields.

### Required Flags

| Flag | Description |
|------|-------------|
| `--domain` | Domain name to verify |

### Examples

```bash
nlm sn domain verify --domain mail.example.com
nlm sn domain verify --domain mail.example.com --format table
```

---

## nlm sn domain remove

Remove a sender domain registration from your account.

### Required Flags

| Flag | Description |
|------|-------------|
| `--domain` | Domain name to remove |

### Examples

```bash
nlm sn domain remove --domain mail.example.com

# Dry-run to preview
nlm sn domain remove --domain mail.example.com --dry-run
```

---

## Domain Setup Workflow

The typical workflow for setting up a sender domain:

```bash
# Step 1: Register the domain and get required DNS records
nlm sn domain create --domain mail.example.com

# Step 2: Add the DNS records at your domain registrar
#   - Add the SPF TXT record
#   - Add the DKIM CNAME record
#   (wait for DNS propagation — up to 48 hours)

# Step 3: Verify the DNS configuration
nlm sn domain verify --domain mail.example.com

# Step 4: Once all records show "valid": true, you can send from this domain
nlm sn email send \
  --subject "Hello" \
  --from-address "noreply@mail.example.com" \
  --html "<p>Sent from a verified domain!</p>" \
  --to user@example.com
```

---

## Notes

- **DNS propagation:** After adding DNS records, it can take up to 48 hours for changes
  to propagate. Run `verify` periodically until all records show `valid: true`.
- **SPF and DKIM:** Both record types must be valid for full verification. Partial
  verification (only SPF or only DKIM) may cause delivery issues.
- **Subdomain recommended:** Use a subdomain like `mail.example.com` rather than the
  root domain to avoid conflicts with existing DNS records.
- The helper command `nlm helper domain-setup` (see **nlm-helper-domain-setup** skill)
  provides a guided interactive flow that combines create, wait, and verify steps.
"#;
