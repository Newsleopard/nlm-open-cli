---
name: persona-customer-success
version: 1.0.0
description: "Customer success: Monitor campaign health, track engagement, and manage subscriber lists."
metadata:
  openclaw:
    category: "persona"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-campaign", "nlm-edm-report", "nlm-edm-contacts"]
---

# Persona — Customer Success

Role-based skill bundle for customer success managers who monitor campaign health,
track subscriber engagement, and generate performance reports.

## Prerequisite Skills

This persona depends on the following skills for full command reference:

- **nlm-edm-campaign** — View, analyze, and compare campaigns
- **nlm-edm-report** — Retrieve campaign reports, click details, and export data
- **nlm-edm-contacts** — Review subscriber lists and engagement segments

## Relevant Workflows

- **recipe-campaign-performance-review** — Post-send performance analysis
- **recipe-contact-cleanup** — Identify and manage inactive subscribers

## Instructions

1. **Start with a recent campaign overview** to see overall performance:

   ```bash
   nlm edm report summary
   ```

2. **Check per-link click details** to understand what content resonates:

   ```bash
   nlm edm report clicks --sn CAM_SN --format table
   ```

3. **Analyze campaigns for optimization suggestions** — the analyze command provides
   actionable recommendations:

   ```bash
   nlm edm campaign analyze --sn CAM_SN
   ```

4. **Compare campaign performance** side by side to identify trends:

   ```bash
   nlm edm campaign compare --sns CAM1 CAM2
   ```

5. **Review top-performing lists** to understand which audience segments are most
   engaged:

   ```bash
   nlm edm contacts top-lists --format table
   ```

6. **Export detailed reports** for stakeholder presentations or deeper analysis:

   ```bash
   nlm helper report-download --sn CAM_SN --output report.csv
   ```

## Tips

- Use `--format table` for quick at-a-glance reviews during team standups.
- Compare campaigns sent to the same list at different times to measure engagement
  trend changes.
- The `analyze` command highlights areas like subject line length, send timing, and
  list health — use these as conversation starters with customers.
- Export reports to CSV for import into spreadsheets or BI tools.
- Monitor bounce and unsubscribe rates closely — rising numbers may indicate list
  hygiene issues that need attention.
