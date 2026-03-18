---
name: recipe-weekly-newsletter
version: 1.0.0
description: "Send a weekly newsletter: prepare content, submit campaign, and review results."
metadata:
  openclaw:
    category: "recipe"
    domain: "campaign"
    requires:
      bins: ["nlm"]
      skills: ["nlm-edm-campaign", "nlm-edm-contacts", "nlm-edm-report"]
---

# Recipe: Weekly Newsletter

End-to-end workflow for sending a weekly newsletter campaign — from preflight
checks through delivery and performance review.

## Prerequisites

- EDM API key configured (`nlm config set edm_api_key "..."`)
- At least one contact list (group) with subscribers
- Newsletter HTML file ready (e.g., `newsletter.html`)
- Verified sender address on the Newsleopard dashboard

---

## Steps

### Step 1 — Check account balance

Verify you have enough email credits before sending.

```bash
nlm edm account balance
```

Confirm the `email` field has enough credits to cover your list size.

### Step 2 — List contact groups

Identify which lists to send to.

```bash
nlm edm contacts list-groups
```

Note the serial numbers (`sn`) of the target lists (e.g., `L1`, `L2`).

### Step 3 — Submit the campaign

Use the `campaign-send` helper to create and send in one step. The `--wait`
flag blocks until the API confirms the campaign is queued.

```bash
nlm helper campaign-send \
  --name 'Weekly Newsletter #42' \
  --lists L1,L2 \
  --subject 'This Week at ACME' \
  --from-name 'ACME News' \
  --from-address news@acme.com \
  --html-file newsletter.html \
  --wait
```

The command outputs the campaign serial number (`CAM_SN`) on success.

### Step 4 — Check campaign status

Confirm the campaign has been delivered or is in progress.

```bash
nlm edm campaign status --sn CAM_SN
```

Possible statuses: `draft`, `queued`, `sending`, `sent`, `paused`, `failed`.

### Step 5 — Review metrics

After delivery completes, review open rate, click rate, and bounces.

```bash
nlm edm report metrics --sns CAM_SN
```

Use `--format table` for a quick summary, or pipe JSON to `jq` for scripting.

---

## Tips

- **Preview first:** Add `--dry-run` to the `campaign-send` command to see the
  exact API request without sending anything.
- **Schedule for later:** Use `--schedule '2025-03-01 09:00'` to queue the
  campaign for a future date/time.
- **Multiple formats:** Append `--format table` to any step for human-readable
  output, or `--format csv` to export data for spreadsheets.
- **Automate:** Chain these steps in a shell script and run via cron for truly
  hands-off weekly sends.
