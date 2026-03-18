---
name: nlm-sn-domain
version: 1.0.0
description: "Surenotify Domains: Register, verify, and remove sender domains."
metadata:
  openclaw:
    category: "group"
    domain: "domain"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-sn"]
---

# nlm sn domain — Sender Domain Verification

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

