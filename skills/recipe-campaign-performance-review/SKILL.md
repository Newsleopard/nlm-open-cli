---
name: recipe-campaign-performance-review
version: 1.0.0
description: "Review campaign performance: list recent campaigns, compare metrics, and export detailed report."
metadata:
  openclaw:
    category: "recipe"
    domain: "report"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-report", "nlm-edm-campaign"]
---

# Recipe: Campaign Performance Review

Audit recent campaign performance — list campaigns in a date range, compare
key metrics, drill into a specific campaign, and export a detailed report.

## Prerequisites

- EDM API key configured
- At least one sent campaign in the date range you want to review

---

## Steps

### Step 1 — List reports for the period

```bash
nlm edm report list --start 2025-01-01 --end 2025-01-31 --format table
```

This shows campaign name, serial number, send date, and summary metrics for
each campaign in the date range.

### Step 2 — Get metrics for multiple campaigns

Compare several campaigns side by side.

```bash
nlm edm report metrics --sns CAM1,CAM2,CAM3 --format table
```

Key columns: `sent`, `delivered`, `open_rate`, `click_rate`, `bounce_rate`,
`unsubscribe_rate`.

### Step 3 — Analyze a specific campaign

Drill into one campaign for detailed insights.

```bash
nlm edm campaign analyze --sn CAM1
```

This returns time-series data (opens/clicks by hour), top clicked links,
device breakdown, and geo distribution.

### Step 4 — Export detailed report

Download the full report as a CSV file for further analysis in a spreadsheet
or BI tool.

```bash
nlm helper report-download --sn CAM1 --output jan-report.csv
```

Note: report export is rate-limited to 1 request per 10 seconds. The helper
handles the wait automatically.

---

## Tips

- **Benchmarking:** Compare `open_rate` and `click_rate` across campaigns to
  identify trends. A declining open rate may indicate list fatigue.
- **Segment analysis:** Export the CSV and filter by domain (gmail.com,
  yahoo.com, etc.) to spot deliverability issues with specific providers.
- **Automate monthly reviews:** Script these four steps and schedule monthly
  to build a performance dashboard over time.
- **JSON for scripting:** Drop `--format table` and pipe JSON to `jq` for
  automated threshold checks (e.g., alert if bounce rate exceeds 5%).
