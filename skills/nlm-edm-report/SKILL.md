---
name: nlm-edm-report
version: 0.1.2
description: "EDM Report: List reports, get metrics, export data, and view click breakdowns."
metadata:
  openclaw:
    category: "group"
    domain: "report"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM Report

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Retrieve campaign reports, metrics, and performance data.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm report list` | List campaign reports by date range |
| `nlm edm report metrics` | Get metrics for one or more campaigns |
| `nlm edm report export` | Export a campaign report (async) |
| `nlm edm report download-link` | Get download link for an exported report |
| `nlm edm report summary` | Recent campaigns performance summary (MCP) |
| `nlm edm report clicks` | Per-link click breakdown (MCP) |

## Parameter Reference

### list

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--start-date` | Yes | Start date (e.g. `2025-01-01`) |
| `--end-date` | Yes | End date (e.g. `2025-01-31`) |

### metrics

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sns` | Yes | Comma-separated campaign SNs |

### export

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |
| `--wait` | No | Wait for the export to complete and download |
| `--output` | No | Output file path (used with `--wait`) |

### download-link

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### summary (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--days` | No | Number of days to look back (default: `30`) |

### clicks (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

## Examples

```bash
# List reports for January 2025
nlm edm report list --start-date 2025-01-01 --end-date 2025-01-31

# Get metrics for multiple campaigns
nlm edm report metrics --sns CAM001,CAM002

# Export a report (triggers async job)
nlm edm report export --sn CAM12345

# Export and wait for download
nlm edm report export --sn CAM12345 --wait --output report.csv

# Get download link for a previously exported report
nlm edm report download-link --sn CAM12345

# Recent campaigns summary (last 7 days)
nlm edm report summary --days 7

# Per-link click breakdown
nlm edm report clicks --sn CAM12345
```

## Notes

- Report export is **rate-limited to 1 request per 10 seconds** — stricter
  than the general 2 req/s limit. The CLI enforces this automatically.
- Export is asynchronous: `export` triggers the job, then use `download-link`
  to retrieve the result, or pass `--wait` to poll automatically.
- MCP commands (`summary`, `clicks`) require an MCP connection (`NL_MCP_URL`).

