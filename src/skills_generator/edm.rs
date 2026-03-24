//! EDM (Email Direct Marketing) skill definitions.
//!
//! Returns 8 skills: 1 service overview + 7 API group skills covering
//! all 20 EDM endpoints plus MCP-backed commands.

use super::{SkillCategory, SkillDefinition};

// ── Skill body constants ────────────────────────────────────────────────────

const EDM_OVERVIEW_BODY: &str = r#"# Newsleopard EDM API

The EDM (Email Direct Marketing) API provides bulk email marketing capabilities
through `api.newsleopard.com`.

## Authentication

All requests require an `x-api-key` header. Configure via:

```bash
nlm config set edm_api_key YOUR_KEY
```

Or set the `NL_EDM_API_KEY` environment variable.

## Rate Limits

- **General:** 2 requests/second (token bucket)
- **Report export:** 1 request/10 seconds (stricter limit)

The CLI enforces these limits automatically with a built-in rate limiter.

## Variable Syntax

EDM content uses `${FIELD_NAME}` for personalization variables (e.g.
`${EMAIL}`, `${NAME}`). Do **not** use the Surenotify `{{variable}}`
syntax in EDM content — the CLI will warn if it detects cross-syntax usage.

## Subcommand Groups

| Group | Description |
|-------|-------------|
| `nlm edm contacts` | Manage contact groups and imports |
| `nlm edm campaign` | Create, send, pause, and analyze campaigns |
| `nlm edm ab-test` | A/B test campaigns (subject, sender, or content) |
| `nlm edm report` | Campaign reports, metrics, and exports |
| `nlm edm template` | List, retrieve, and save templates |
| `nlm edm automation` | Trigger automation workflows |
| `nlm edm account` | Account balance and credits |

## Global Flags

All EDM commands support these flags (see `nlm-shared` skill for details):

- `--format json|table|yaml|csv` — output format
- `--dry-run` — preview the request without sending
- `--profile NAME` — use a named config profile
- `--verbose` — enable debug logging

## Examples

```bash
# List contact groups as a table
nlm edm contacts list-groups --format table

# Submit a campaign
nlm edm campaign submit --name 'March Newsletter' --lists L1 \
  --subject 'March Updates' --from-name 'ACME' \
  --from-address news@acme.com --html-file newsletter.html

# Check account balance
nlm edm account balance
```
"#;

const EDM_CONTACTS_BODY: &str = r#"# EDM Contacts

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
"#;

const EDM_CAMPAIGN_BODY: &str = r#"# EDM Campaign

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Create, send, manage, and analyze email campaigns.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm campaign submit` | Submit a campaign for sending |
| `nlm edm campaign submit-once` | One-time campaign from a contacts file |
| `nlm edm campaign delete` | Delete one or more campaigns |
| `nlm edm campaign pause` | Pause a sending campaign |
| `nlm edm campaign status` | Check campaign sending status |
| `nlm edm campaign analyze` | AI-powered performance analysis (MCP) |
| `nlm edm campaign compare` | Compare 2-5 campaigns side by side (MCP) |
| `nlm edm campaign preflight` | Pre-send validation check (MCP) |
| `nlm edm campaign find` | Search campaigns by keyword (MCP) |
| `nlm edm campaign best-time` | Best send time recommendation (MCP) |

## Parameter Reference

### submit

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--name` | Yes | Campaign name |
| `--lists` | Yes | Comma-separated list SNs to send to |
| `--subject` | Yes | Email subject line |
| `--from-name` | Yes | Sender display name |
| `--from-address` | Yes | Sender email address |
| `--html` | No* | HTML content as inline string |
| `--html-file` | No* | Path to an HTML file |
| `--footer-lang` | No | Footer language: `chinese`, `english`, `japanese` (default: `chinese`) |
| `--preheader` | No | Preheader text |
| `--exclude-lists` | No | Comma-separated list SNs to exclude |
| `--schedule` | No | `immediate` (default) or `scheduled` |
| `--schedule-date` | No | Schedule date (e.g. `2025-03-20T09:00:00`) |
| `--schedule-timezone` | No | Timezone offset (e.g. `8` for UTC+8) |
| `--ga` | No | Enable Google Analytics tracking |
| `--ga-ecommerce` | No | Enable GA e-commerce tracking |
| `--utm-campaign` | No | Custom utm_campaign value |
| `--utm-content` | No | Custom utm_content value |

*One of `--html` or `--html-file` is required (mutually exclusive).

### submit-once

Same parameters as `submit` except `--lists` and `--exclude-lists` are
replaced by:

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--contacts-file` | Yes | CSV/Excel file containing recipient contacts |

### delete

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sns` | Yes | Comma-separated campaign SNs to delete |

### pause

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### status

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### analyze (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### compare (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sns` | Yes | 2-5 campaign SNs to compare |

### preflight (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--sn` | Yes | Campaign SN |

### find (MCP)

| Parameter | Required | Description |
|-----------|----------|-------------|
| `<query>` | Yes | Search query (positional argument) |

### best-time (MCP)

No parameters required.

## Examples

```bash
# Send immediately to list L1
nlm edm campaign submit --name 'March Newsletter' --lists L1 \
  --subject 'March Updates' --from-name 'ACME' \
  --from-address news@acme.com --html-file newsletter.html

# Schedule for a specific time
nlm edm campaign submit --name 'Promo' --lists L1,L2 --exclude-lists L3 \
  --subject 'Sale!' --from-name 'Shop' --from-address shop@acme.com \
  --html '<h1>50% Off</h1>' --schedule scheduled \
  --schedule-date '2025-03-20T09:00:00' --schedule-timezone 8

# Dry-run to preview the request
nlm edm campaign submit --name Test --lists L1 --subject Hi \
  --from-name Me --from-address me@x.com --html '<p>hi</p>' --dry-run

# One-time campaign from file
nlm edm campaign submit-once --contacts-file contacts.csv \
  --name 'One-time Blast' --subject 'Flash Sale' \
  --from-name Shop --from-address shop@acme.com --html-file promo.html

# Delete campaigns
nlm edm campaign delete --sns CAM001,CAM002

# Pause a sending campaign
nlm edm campaign pause --sn CAM12345

# Check campaign status
nlm edm campaign status --sn CAM12345

# AI analysis
nlm edm campaign analyze --sn CAM12345

# Compare campaigns
nlm edm campaign compare --sns CAM001 CAM002 CAM003

# Pre-flight check
nlm edm campaign preflight --sn CAM12345

# Search campaigns
nlm edm campaign find "March newsletter"

# Best send time
nlm edm campaign best-time
```

## Notes

- EDM uses `${FIELD_NAME}` variable syntax in subject and content. The CLI
  will warn if it detects Surenotify `{{variable}}` syntax.
- MCP commands (`analyze`, `compare`, `preflight`, `find`, `best-time`)
  require an MCP connection configured via `NL_MCP_URL`.
- The `compare` command accepts 2 to 5 campaign SNs.
"#;

const EDM_AB_TEST_BODY: &str = r#"# EDM A/B Test

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Submit A/B test campaigns to optimize subject lines, sender identity, or content.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm ab-test submit` | Submit an A/B test campaign |
| `nlm edm ab-test submit-once` | One-time A/B test from a contacts file |

## Parameter Reference

### submit / submit-once

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--name` | Yes | Campaign name |
| `--lists` | Yes | Comma-separated list SNs to send to |
| `--test-on` | Yes | What to test: `subject`, `sender`, or `content` |
| `--proportion` | Yes | Percentage of recipients for test phase (e.g. `20`) |
| `--test-duration` | Yes | Duration of the test phase |
| `--test-unit` | Yes | Unit for duration: `hours` or `days` |

**Subject test fields** (when `--test-on subject`):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--subject-a` | Yes | Subject line for variant A |
| `--subject-b` | Yes | Subject line for variant B |
| `--from-name` | Yes | Sender display name |
| `--from-address` | Yes | Sender email address |
| `--html` / `--html-file` | Yes | Shared HTML content |

**Sender test fields** (when `--test-on sender`):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--from-name-a` | Yes | Sender name for variant A |
| `--from-address-a` | Yes | Sender address for variant A |
| `--from-name-b` | Yes | Sender name for variant B |
| `--from-address-b` | Yes | Sender address for variant B |
| `--subject` | Yes | Shared subject line |
| `--html` / `--html-file` | Yes | Shared HTML content |

**Content test fields** (when `--test-on content`):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--html-content-a-file` | Yes | HTML file for variant A |
| `--html-content-b-file` | Yes | HTML file for variant B |
| `--subject` | Yes | Shared subject line |
| `--from-name` | Yes | Sender display name |
| `--from-address` | Yes | Sender email address |

**Common optional fields** (all test types):

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--footer-lang` | No | Footer language: `chinese`, `english`, `japanese` (default: `chinese`) |
| `--preheader` | No | Preheader text |
| `--exclude-lists` | No | Comma-separated list SNs to exclude |
| `--schedule` | No | `immediate` (default) or `scheduled` |
| `--schedule-date` | No | Schedule date (e.g. `2025-03-20T09:00:00`) |
| `--schedule-timezone` | No | Timezone offset (e.g. `8` for UTC+8) |
| `--ga` | No | Enable Google Analytics tracking |
| `--ga-ecommerce` | No | Enable GA e-commerce tracking |
| `--utm-campaign` | No | Custom utm_campaign value |
| `--utm-content` | No | Custom utm_content value |

**submit-once only:**

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--contacts-file` | Yes | CSV/Excel file containing recipient contacts |

## Examples

```bash
# Test two subject lines on 20% of list, pick winner after 4 hours
nlm edm ab-test submit --name 'Subject Test' --lists L1 \
  --test-on subject --subject-a 'Free Shipping' --subject-b '50% Off' \
  --from-name ACME --from-address news@acme.com --html-file email.html \
  --proportion 20 --test-duration 4 --test-unit hours

# Test two content versions from a one-time contacts file
nlm edm ab-test submit-once --contacts-file contacts.csv \
  --name 'Content Test' --test-on content \
  --html-content-a-file version_a.html --html-content-b-file version_b.html \
  --subject 'Newsletter' --from-name ACME --from-address news@acme.com \
  --proportion 30 --test-duration 1 --test-unit days

# Dry-run to preview
nlm edm ab-test submit --name 'Sender Test' --lists L1 \
  --test-on sender \
  --from-name-a 'Sales Team' --from-address-a sales@acme.com \
  --from-name-b 'ACME News' --from-address-b news@acme.com \
  --subject 'Check this out' --html-file email.html \
  --proportion 25 --test-duration 6 --test-unit hours --dry-run
```

## Notes

- The test phase sends to `--proportion`% of recipients. After `--test-duration`,
  the winning variant is automatically sent to the remaining recipients.
- Winner is selected based on open rate by default.
- Required variant fields depend on the `--test-on` value — the CLI validates
  this at parse time.
"#;

const EDM_REPORT_BODY: &str = r#"# EDM Report

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
"#;

const EDM_TEMPLATE_BODY: &str = r#"# EDM Template

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
"#;

const EDM_AUTOMATION_BODY: &str = r#"# EDM Automation

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
"#;

const EDM_ACCOUNT_BODY: &str = r#"# EDM Account

> **Prerequisites:** `nlm-shared` (global flags, output formats), `nlm-edm` (auth, rate limits)

Check account balance and credit information.

## Commands

| Command | Description |
|---------|-------------|
| `nlm edm account balance` | Show email and SMS credits |

## Parameter Reference

### balance

No parameters required.

## Examples

```bash
# Check account balance (JSON)
nlm edm account balance

# Check as a table
nlm edm account balance --format table

# Check in YAML format
nlm edm account balance --format yaml
```

## Notes

- Returns remaining email sends and SMS credits for the account.
- Useful for monitoring usage before launching large campaigns.
"#;

// ── Public API ──────────────────────────────────────────────────────────────

/// Returns all 8 EDM skill definitions.
pub fn skills() -> Vec<SkillDefinition> {
    let version = "0.1.2".to_string();
    let bin = "nlm".to_string();

    vec![
        // 1. Service overview
        SkillDefinition {
            name: "nlm-edm".to_string(),
            version: version.clone(),
            description: "Newsleopard EDM API: Bulk email marketing \
                — campaigns, contacts, reports, templates, A/B tests, \
                automations, and account (20 endpoints)."
                .to_string(),
            category: SkillCategory::Service,
            domain: None,
            requires_bins: vec![bin.clone()],
            requires_skills: vec![],
            body: EDM_OVERVIEW_BODY.to_string(),
        },
        // 2. Contacts
        SkillDefinition {
            name: "nlm-edm-contacts".to_string(),
            version: version.clone(),
            description: "EDM Contacts: Create groups, import contacts, \
                check import status, and remove by filter."
                .to_string(),
            category: SkillCategory::Group,
            domain: Some("contacts".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_CONTACTS_BODY.to_string(),
        },
        // 3. Campaign
        SkillDefinition {
            name: "nlm-edm-campaign".to_string(),
            version: version.clone(),
            description: "EDM Campaign: Submit, schedule, pause, delete, \
                and analyze email campaigns."
                .to_string(),
            category: SkillCategory::Group,
            domain: Some("campaign".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_CAMPAIGN_BODY.to_string(),
        },
        // 4. A/B Test
        SkillDefinition {
            name: "nlm-edm-ab-test".to_string(),
            version: version.clone(),
            description: "EDM A/B Test: Submit A/B test campaigns \
                comparing subject lines, senders, or content."
                .to_string(),
            category: SkillCategory::Group,
            domain: Some("ab-test".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_AB_TEST_BODY.to_string(),
        },
        // 5. Report
        SkillDefinition {
            name: "nlm-edm-report".to_string(),
            version: version.clone(),
            description: "EDM Report: List reports, get metrics, \
                export data, and view click breakdowns."
                .to_string(),
            category: SkillCategory::Group,
            domain: Some("report".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_REPORT_BODY.to_string(),
        },
        // 6. Template
        SkillDefinition {
            name: "nlm-edm-template".to_string(),
            version: version.clone(),
            description: "EDM Template: List, retrieve, and save \
                email templates."
                .to_string(),
            category: SkillCategory::Group,
            domain: Some("template".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_TEMPLATE_BODY.to_string(),
        },
        // 7. Automation
        SkillDefinition {
            name: "nlm-edm-automation".to_string(),
            version: version.clone(),
            description: "EDM Automation: Trigger automation workflows \
                with recipient targeting."
                .to_string(),
            category: SkillCategory::Group,
            domain: Some("automation".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_AUTOMATION_BODY.to_string(),
        },
        // 8. Account
        SkillDefinition {
            name: "nlm-edm-account".to_string(),
            version: version.clone(),
            description: "EDM Account: Check email and SMS credit balance.".to_string(),
            category: SkillCategory::Group,
            domain: Some("account".to_string()),
            requires_bins: vec![bin.clone()],
            requires_skills: vec!["nlm-shared".to_string(), "nlm-edm".to_string()],
            body: EDM_ACCOUNT_BODY.to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_8_skills() {
        let s = skills();
        assert_eq!(s.len(), 8, "Expected 8 EDM skills, got {}", s.len());
    }

    #[test]
    fn first_skill_is_service_overview() {
        let s = skills();
        assert_eq!(s[0].name, "nlm-edm");
        assert!(matches!(s[0].category, SkillCategory::Service));
        assert!(s[0].domain.is_none());
        assert!(s[0].requires_skills.is_empty());
    }

    #[test]
    fn group_skills_have_correct_requires() {
        let s = skills();
        for skill in &s[1..] {
            assert!(
                matches!(skill.category, SkillCategory::Group),
                "{} should be Group category",
                skill.name
            );
            assert!(
                skill.requires_skills.contains(&"nlm-shared".to_string()),
                "{} should require nlm-shared",
                skill.name
            );
            assert!(
                skill.requires_skills.contains(&"nlm-edm".to_string()),
                "{} should require nlm-edm",
                skill.name
            );
            assert!(
                skill.domain.is_some(),
                "{} should have a domain",
                skill.name
            );
        }
    }

    #[test]
    fn all_names_start_with_nlm_edm() {
        for skill in skills() {
            assert!(
                skill.name.starts_with("nlm-edm"),
                "Skill name '{}' should start with 'nlm-edm'",
                skill.name
            );
        }
    }

    #[test]
    fn bodies_are_non_empty() {
        for skill in skills() {
            assert!(
                !skill.body.is_empty(),
                "Skill '{}' should have a non-empty body",
                skill.name
            );
            assert!(
                skill.body.contains('#'),
                "Skill '{}' body should contain markdown headings",
                skill.name
            );
        }
    }
}
