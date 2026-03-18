---
name: nlm-helper-report-download
version: 1.0.0
description: "nlm helper: Export a campaign report and download the file — combines export + poll + download."
metadata:
  openclaw:
    category: "helper"
    domain: "report"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm", "nlm-edm-report"]
---

# nlm helper report-download

> **Prerequisite skills:** `nlm-shared`, `nlm-edm`, `nlm-edm-report`

Export a campaign report and download the resulting file. This helper combines
the export trigger, download-link polling, and file download into one command.

## Usage

```bash
nlm helper report-download --sn <CAMPAIGN_SN> --output <PATH>
```

Alias: `nlm x report-download ...`

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `--sn` | Yes | — | Campaign SN to export the report for |
| `--output` | Yes | — | Output file path (e.g. `report.csv`) |

## Workflow

1. **Trigger export** — `POST /v1/report/{sn}/export`.
2. **Poll download link** — spinner polls every 10 s for up to 600 s.
3. **Download file** — saves the CSV to the specified output path.
4. **Return summary** — JSON with status, path, and file size.

## Examples

```bash
# Download a campaign report
nlm helper report-download --sn CAM12345 --output ./march-report.csv

# Using the short alias
nlm x report-download --sn CAM12345 --output report.csv
```

## Tips

- Report export uses a separate, stricter rate limiter: **1 request per 10 s**.
  This is handled automatically by the polling interval.
- The output file is overwritten if it already exists.
- The command returns a JSON summary to stdout:
  ```json
  {
    "status": "downloaded",
    "path": "./march-report.csv",
    "size": 45230
  }
  ```
- If the export times out (600 s), exit code 4 (Network/Timeout) is returned.

