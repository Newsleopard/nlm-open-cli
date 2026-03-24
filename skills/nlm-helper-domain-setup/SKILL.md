---
name: nlm-helper-domain-setup
version: 1.0.0
description: "nlm helper: Set up a sender domain with DNS records and optional auto-verify."
metadata:
  openclaw:
    category: "helper"
    domain: "domain"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-sn", "nlm-sn-domain"]
---

# nlm helper domain-setup

> **Prerequisite skills:** `nlm-shared`, `nlm-sn`, `nlm-sn-domain`

Set up a sender domain with DNS verification records and optional automatic
verification after a waiting period.

## Usage

```bash
nlm helper domain-setup --domain <DOMAIN> [--auto-verify-after <SECONDS>]
```

Alias: `nlm x domain-setup ...`

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `--domain` | Yes | — | Domain to set up (e.g. `mail.example.com`) |
| `--auto-verify-after` | No | — | Seconds to wait before attempting automatic DNS verification |

## Workflow

1. **Create domain** — registers the domain via the Surenotify API.
2. **Display DNS records** — prints the required TXT and CNAME records to
   stderr for you to configure in your DNS provider.
3. **(if `--auto-verify-after`)** **Wait** — spinner counts down the specified
   seconds for DNS propagation.
4. **(if `--auto-verify-after`)** **Verify** — triggers domain verification
   and returns the result.

## Examples

```bash
# Manual verification (shows DNS records, then exits)
nlm helper domain-setup --domain mail.example.com

# Auto-verify after 5 minutes (300 s) of DNS propagation
nlm x domain-setup --domain mail.example.com --auto-verify-after 300

# Auto-verify after 1 minute (quick check)
nlm helper domain-setup --domain mail.example.com --auto-verify-after 60
```

## Tips

- Without `--auto-verify-after`, the command returns the DNS records and exits.
  You can verify later with `nlm sn domain verify --domain <DOMAIN>`.
- DNS propagation typically takes 1-5 minutes but can take up to 48 hours
  depending on your DNS provider and TTL settings.
- The DNS records printed to stderr include record type (TXT/CNAME), name,
  and value. Configure all of them before verifying.
- This command uses the **Surenotify API** (not EDM), so it requires
  `NL_SN_API_KEY` to be configured.

