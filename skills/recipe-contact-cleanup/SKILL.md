---
name: recipe-contact-cleanup
version: 1.0.0
description: "Clean up contact lists: identify low-engagement contacts and remove bounced addresses."
metadata:
  openclaw:
    category: "recipe"
    domain: "contacts"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-contacts", "nlm-edm-report"]
---

# Recipe: Contact Cleanup

Clean up contact lists by identifying low-engagement contacts, reviewing
bounced addresses from campaign reports, and removing invalid entries.

## Prerequisites

- EDM API key configured
- At least one contact list with historical campaign data
- Recent campaign reports available for bounce analysis

---

## Steps

### Step 1 — Check top lists

Review your largest and most active contact lists.

```bash
nlm edm contacts top-lists --format table
```

Identify lists that may need cleanup based on size and last-send date.

### Step 2 — Review campaign bounces

Pull bounce data from recent campaign reports to identify invalid addresses.

```bash
nlm edm report metrics --sns CAM1 --format table
```

Check the `bounce_rate` field. If it exceeds 2-3%, the list needs cleanup.
For detailed bounce data, export the report:

```bash
nlm helper report-download --sn CAM1 --output bounces.csv
```

Filter the CSV for rows with `status = bounced` to get the list of addresses
to remove.

### Step 3 — Remove bounced contacts

Remove a specific bounced address from a list.

```bash
nlm edm contacts remove \
  --list-sn L1 \
  --field email \
  --op eq \
  --value bounced@example.com
```

For bulk removal, script this step by looping over the bounced addresses
extracted from the CSV:

```bash
while IFS=, read -r email _rest; do
  nlm edm contacts remove --list-sn L1 --field email --op eq --value "$email"
done < bounced-emails.txt
```

---

## Tips

- **Bounce types:** Hard bounces (invalid address, domain does not exist)
  should be removed immediately. Soft bounces (mailbox full, server
  temporarily unavailable) may resolve on their own — retry before removing.
- **Regular cadence:** Run this cleanup monthly to maintain list hygiene and
  protect your sender reputation.
- **Rate limits:** The EDM API allows 2 requests per second. The bulk removal
  loop above naturally stays within this limit, but for very large lists
  consider adding a small delay.
- **Backup first:** Export the full contact list before removing entries so
  you can restore if needed:
  `nlm edm contacts list --list-sn L1 --format csv > backup.csv`
