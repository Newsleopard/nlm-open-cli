---
name: recipe-domain-migration
version: 1.0.0
description: "Migrate sender domain: set up new domain, verify DNS, then remove old domain."
metadata:
  openclaw:
    category: "recipe"
    domain: "domain"
    requires:
      bins: ["nlm"]
      skills: ["nlm-sn-domain"]
---

# Recipe: Domain Migration

Migrate from an old sender domain to a new one — create the new domain,
verify DNS records, confirm it works, then remove the old domain.

## Prerequisites

- Surenotify API key configured
- Access to DNS management for the new domain
- Knowledge of the old domain to be removed

---

## Steps

### Step 1 — Create the new domain

```bash
nlm sn domain create --domain new.example.com
```

The output displays the required DNS records (CNAME, TXT) that must be added
to your DNS provider.

### Step 2 — Configure DNS records

Add the displayed records to your DNS provider:

- **CNAME record:** Points the sending subdomain to Surenotify's mail servers.
- **TXT record:** SPF/DKIM verification for email authentication.

Wait for DNS propagation (typically a few minutes, up to 48 hours).

### Step 3 — Verify the new domain

```bash
nlm sn domain verify --domain new.example.com
```

If verification fails, double-check the DNS records with `dig` and retry.
The API returns specific error messages indicating which records are missing
or incorrect.

### Step 4 — Remove the old domain

Once the new domain is verified and tested, remove the old one.

```bash
nlm sn domain remove --domain old.example.com
```

---

## Tips

- **Test before removing:** Send a test email via the new domain
  (`nlm sn email send --from-address noreply@new.example.com ...`) and
  confirm delivery before removing the old domain.
- **Gradual migration:** If you have high volume, consider running both
  domains in parallel for a transition period to catch any issues.
- **DNS TTL:** Lower the TTL on old DNS records before migration to speed up
  the cutover. Restore normal TTL values after the migration is complete.
- **Update templates:** After migration, update all email templates and
  application code to use the new `from-address` domain.
