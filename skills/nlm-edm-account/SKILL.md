---
name: nlm-edm-account
version: 0.1.2
description: "EDM Account: Check email and SMS credit balance."
metadata:
  openclaw:
    category: "group"
    domain: "account"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM Account

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Check account balance and credit information.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm account balance` | Show email and SMS credits |

## Parameter Reference

### balance

No parameters required.

## Examples

```bash
# Check account balance (JSON)
nlm edm account balance

# Check as a table
nlm edm account balance --format table

# Check in YAML format
nlm edm account balance --format yaml
```

## Notes

- Returns remaining email sends and SMS credits for the account.
- Useful for monitoring usage before launching large campaigns.

