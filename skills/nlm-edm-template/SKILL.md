---
name: nlm-edm-template
version: 0.1.2
description: "EDM Template: List, retrieve, and save email templates."
metadata:
  openclaw:
    category: "group"
    domain: "template"
    requires:
      bins: ["nlm"]
      skills: ["nlm-shared", "nlm-edm"]
---

# EDM Template

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

List, retrieve, and save email templates.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm template list` | List all templates |
| `nlm edm template get` | Get a template by ID |
| `nlm edm template save` | Save a campaign as a reusable template (MCP) |

## Parameter Reference

### list

No parameters required.

### get

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--id` | Yes | Template ID |
| `--output` | No | Save template HTML to this file path |

### save (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--campaign-sn` | Yes | Campaign SN to save as template |
| `--name` | Yes | Template name |

## Examples

```bash
# List all templates
nlm edm template list

# List templates as a table
nlm edm template list --format table

# Get a template by ID
nlm edm template get --id TPL001

# Get a template and save to file
nlm edm template get --id TPL001 --output template.html

# Save a campaign as a reusable template
nlm edm template save --campaign-sn CAM12345 --name 'Monthly Newsletter Template'
```

## Notes

- The `save` command requires an MCP connection (`NL_MCP_URL`).
- Templates saved via MCP are available for future campaigns.

