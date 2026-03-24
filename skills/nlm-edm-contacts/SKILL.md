---
name: nlm-edm-contacts
version: 0.1.2
description: "EDM Contacts: Create groups, import contacts, check import status, and remove by filter."
metadata:
  openclaw:
    category: "group"
    domain: "contacts"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM Contacts

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Manage contact groups and imports for the Newsleopard EDM API.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm contacts create-group` | Create a new contact group |
| `nlm edm contacts list-groups` | List all contact groups |
| `nlm edm contacts import-file` | Import contacts from a CSV/Excel file |
| `nlm edm contacts import-text` | Import contacts from inline CSV text |
| `nlm edm contacts import-status` | Check the status of an import job |
| `nlm edm contacts remove` | Remove contacts matching a filter |
| `nlm edm contacts top-lists` | Top-performing lists by engagement (MCP) |

## Parameter Reference

### create-group

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--name` | Yes | Group name |

### list-groups

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--page` | No | Page number (1-based) |
| `--size` | No | Page size |
| `--page-all` | No | Fetch all pages as NDJSON |

### import-file

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--list-sn` | Yes | Target contact list SN |
| `--file` | Yes | CSV or Excel file path |
| `--webhook-url` | No | Webhook URL for completion notification |
| `--wait` | No | Poll until the import completes |
| `--poll-interval` | No | Seconds between status polls (with `--wait`) |

### import-text

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--list-sn` | Yes | Target contact list SN |
| `--csv-text` | No* | Inline CSV text |
| `--csv-file` | No* | Path to CSV file to read as text body |
| `--webhook-url` | No | Webhook URL for completion notification |

*One of `--csv-text` or `--csv-file` is required (mutually exclusive).

### import-status

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--import-sn` | Yes | Import job SN |

### remove

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--list-sn` | Yes | Target contact list SN |
| `--field` | Yes | Field to filter on (e.g. `email`, `name`) |
| `--operator` | Yes | Comparison operator (e.g. `eq`, `contains`) |
| `--value` | Yes | Value to match |

### top-lists (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--limit` | No | Maximum number of lists to return |

## Examples

```bash
# Create a new contact group
nlm edm contacts create-group --name 'VIP Customers'

# List all groups as a table
nlm edm contacts list-groups --format table

# Stream all groups as NDJSON
nlm edm contacts list-groups --page-all

# Import from CSV file and wait for completion
nlm edm contacts import-file --list-sn L1 --file contacts.csv --wait

# Import from inline CSV
nlm edm contacts import-text --list-sn L1 \
  --csv-text 'email,name\na@b.com,Alice\nc@d.com,Bob'

# Check import job status
nlm edm contacts import-status --import-sn IMP12345

# Remove a specific contact by email
nlm edm contacts remove --list-sn L1 --field email \
  --operator eq --value old@example.com

# Top-performing lists
nlm edm contacts top-lists --limit 5
```

## Notes

- Import operations are asynchronous. Use `--wait` with `import-file` or
  poll manually with `import-status` to track progress.
- The `top-lists` command requires an MCP connection (`NL_MCP_URL`).

