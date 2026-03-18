---
name: nlm-helper-import-and-wait
version: 1.0.0
description: "nlm helper: Import contacts from a file and poll until the import completes."
metadata:
  openclaw:
    category: "helper"
    domain: "contacts"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm", "nlm-edm-contacts"]
---

# nlm helper import-and-wait

> **Prerequisite skills:** `nlm-shared`, `nlm-edm`, `nlm-edm-contacts`

Import contacts from a CSV or Excel file into a contact list and poll until
the import completes, showing a progress spinner.

## Usage

```bash
nlm helper import-and-wait \
  --list-sn <LIST_SN> --file <PATH> \
  [--timeout <SECONDS>] [--poll-interval <SECONDS>]
```

Alias: `nlm x import-and-wait ...`

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `--list-sn` | Yes | — | Contact list SN to import into |
| `--file` | Yes | — | Path to CSV or Excel file to import |
| `--timeout` | No | `600` | Maximum seconds to wait for import completion |
| `--poll-interval` | No | `5` | Seconds between status polls |

## Workflow

1. **Upload file** — sends the file to the EDM import API.
2. **Poll status** — shows a spinner while polling at the configured interval.
3. **Return result** — final import status with counts (success, duplicate,
   failed).

## Examples

```bash
# Import and wait with defaults (600s timeout, 5s polls)
nlm helper import-and-wait --list-sn GRP-001 --file contacts.csv

# Custom timeout and polling interval
nlm x import-and-wait \
  --list-sn GRP-001 \
  --file customers.xlsx \
  --timeout 300 \
  --poll-interval 10

# Table output for human-readable result
nlm helper import-and-wait \
  --list-sn GRP-001 --file contacts.csv --format table
```

## Tips

- The progress spinner displays on stderr; JSON result goes to stdout.
- If the import times out, exit code 4 (Network/Timeout) is returned.
- If the import fails (ERROR status), exit code 1 (Api) is returned.
- The file must be CSV or Excel format. Column mapping follows the target
  list's field configuration.

