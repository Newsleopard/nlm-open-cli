---
name: persona-email-marketer
version: 1.0.0
description: "Email marketer: Plan campaigns, manage audiences, test content, and analyze performance."
metadata:
  openclaw:
    category: "persona"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-campaign", "nlm-edm-contacts", "nlm-edm-report", "nlm-edm-template", "nlm-edm-ab-test"]
---

# Persona — Email Marketer

Role-based skill bundle for email marketers who plan campaigns, manage audiences,
test content variations, and analyze performance metrics.

## Prerequisite Skills

This persona depends on the following skills for full command reference:

- **nlm-edm-campaign** — Create, update, send, analyze, and compare campaigns
- **nlm-edm-contacts** — Manage contact lists, groups, and subscriber segments
- **nlm-edm-report** — Retrieve campaign reports, click details, and export data
- **nlm-edm-template** — Browse, create, and manage email templates
- **nlm-edm-ab-test** — Set up and evaluate A/B tests

## Relevant Workflows

- **recipe-weekly-newsletter** — End-to-end weekly newsletter workflow
- **recipe-ab-test-subject** — Subject line A/B testing workflow
- **recipe-campaign-performance-review** — Post-send performance analysis

## Instructions

1. **Check your account balance** before planning a send:

   ```bash
   nlm edm account balance
   ```

2. **Review audience segments** to choose the right target list:

   ```bash
   nlm edm contacts list-groups
   nlm edm contacts top-lists
   ```

3. **Browse existing templates** for reuse or inspiration:

   ```bash
   nlm edm template list
   ```

4. **Always preview with `--dry-run`** before submitting campaigns — this shows the
   exact HTTP request without executing it:

   ```bash
   nlm edm campaign send 12345 --dry-run
   ```

5. **Use A/B testing for subject line optimization** — split your audience and test
   two subject lines to find the higher-performing variant before sending to the
   full list.

6. **Analyze results after sending** to identify what worked:

   ```bash
   nlm edm campaign analyze --sn CAM_SN
   ```

   Review open rates, click rates, and engagement metrics to refine future campaigns.

7. **Use `--format table` for quick visual scans** of lists and reports:

   ```bash
   nlm edm contacts list --format table
   nlm edm report summary --format table
   ```

## Tips

- Combine `top-lists` output with campaign targeting to focus on your most engaged segments.
- After an A/B test concludes, review the winning variant's metrics to build a library
  of high-performing subject line patterns.
- Schedule sends during peak engagement windows identified in previous report analyses.
- Use `--dry-run` liberally — it costs nothing and prevents accidental sends.
