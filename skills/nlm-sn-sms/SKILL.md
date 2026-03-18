---
name: nlm-sn-sms
version: 1.0.0
description: "Surenotify SMS: Send SMS messages and query delivery events."
metadata:
  openclaw:
    category: "group"
    domain: "sms"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-sn"]
---

# nlm sn sms — SMS Messaging

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

