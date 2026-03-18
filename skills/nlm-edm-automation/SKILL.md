---
name: nlm-edm-automation
version: 0.1.2
description: "EDM Automation: Trigger automation workflows with recipient targeting."
metadata:
  openclaw:
    category: "group"
    domain: "automation"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM Automation

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Trigger automation workflows in the Newsleopard EDM system.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm automation trigger` | Trigger an automation workflow |

## Parameter Reference

### trigger

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--workflow` | Yes | Workflow identifier |
| `--event` | Yes | Event name to trigger |
| `--recipients` | No* | Comma-separated recipient emails |
| `--recipients-file` | No* | File with recipient emails (one per line) |

*One of `--recipients` or `--recipients-file` is required (mutually exclusive).

## Examples

```bash
# Trigger a welcome automation for specific recipients
nlm edm automation trigger --workflow AUTO001 --event welcome \
  --recipients user@example.com,new@example.com

# Trigger from a recipients file
nlm edm automation trigger --workflow AUTO001 --event onboarding \
  --recipients-file new_signups.txt

# Dry-run to preview
nlm edm automation trigger --workflow AUTO001 --event welcome \
  --recipients user@example.com --dry-run
```

## Notes

- Automation workflows must be pre-configured in the Newsleopard dashboard
  before they can be triggered via the CLI.
- The `--recipients` and `--recipients-file` flags are mutually exclusive.

