---
name: recipe-import-and-send
version: 1.0.0
description: "Import a contact list from CSV, wait for completion, then send a campaign to the new list."
metadata:
  openclaw:
    category: "recipe"
    domain: "contacts"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-contacts", "nlm-edm-campaign"]
---

# Recipe: Import Contacts and Send

Import a contact list from a CSV file, wait for the import to finish, then
immediately send a campaign to the newly created list.

## Prerequisites

- EDM API key configured
- A CSV file with at least an `email` column (e.g., `contacts.csv`)
- HTML content file for the campaign
- Verified sender address

---

## Steps

### Step 1 — Create a new contact group

```bash
nlm edm contacts create-group --name 'March Promo'
```

Note the new group serial number (`NEW_SN`) from the output.

### Step 2 — Import contacts and wait

The `import-and-wait` helper uploads the CSV file and polls until the import
job completes.

```bash
nlm helper import-and-wait --list-sn NEW_SN --file contacts.csv
```

The helper reports progress (records processed, duplicates, errors) and exits
once the import is fully complete.

### Step 3 — Send the campaign

```bash
nlm helper campaign-send \
  --name 'March Promo' \
  --lists NEW_SN \
  --subject 'Special Offer' \
  --from-name 'ACME Sales' \
  --from-address sales@acme.com \
  --html-file promo.html \
  --wait
```

---

## Tips

- **CSV format:** The first row must be headers. At minimum, include `email`.
  Optional columns: `name`, `phone`, custom fields defined in your account.
- **Deduplication:** The API automatically deduplicates against existing
  contacts in the group. Duplicate rows in the CSV are counted but not
  imported twice.
- **Large files:** For files over 100,000 rows, the import may take several
  minutes. The `--wait` flag on `import-and-wait` handles this automatically.
- **Dry-run the send:** Add `--dry-run` to the `campaign-send` step to verify
  the request payload before committing.
