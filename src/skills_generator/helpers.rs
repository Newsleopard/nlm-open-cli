//! Skill definitions for the 4 helper/orchestration workflows
//! (`nlm helper` / `nlm x`).

use super::{SkillCategory, SkillDefinition};

// ── Body content ────────────────────────────────────────────────

const CAMPAIGN_SEND_BODY: &str = r#"# nlm helper campaign-send

> **Prerequisite skills:** `nlm-shared`, `nlm-edm`, `nlm-edm-campaign`

Submit a campaign and optionally wait for completion. This helper combines
balance check, campaign submit, status polling, and final metrics retrieval
into a single command.

## Usage

```bash
nlm helper campaign-send \
  --name <NAME> --lists <LIST_SNs> --subject <SUBJECT> \
  --from-name <NAME> --from-address <EMAIL> \
  (--html <HTML> | --html-file <PATH>) \
  [--wait] [options...]
```

Alias: `nlm x campaign-send ...`

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `--name` | Yes | — | Campaign name |
| `--lists` | Yes | — | Comma-separated contact list SNs to send to |
| `--subject` | Yes | — | Email subject line (supports `${FIELD}` variables) |
| `--from-name` | Yes | — | Sender display name |
| `--from-address` | Yes | — | Sender email address |
| `--html` | One of html/html-file | — | Inline HTML content |
| `--html-file` | One of html/html-file | — | Path to an HTML file |
| `--footer-lang` | No | `chinese` | Footer language: `chinese`, `english`, `japanese` |
| `--preheader` | No | — | Preheader text (inbox preview) |
| `--exclude-lists` | No | — | Comma-separated list SNs to exclude |
| `--schedule` | No | `immediate` | `immediate` or `scheduled` |
| `--schedule-date` | No | — | Schedule date (e.g. `2025-01-15T09:00:00`) |
| `--schedule-timezone` | No | — | Timezone offset (e.g. `8` for UTC+8) |
| `--ga` | No | `false` | Enable Google Analytics tracking |
| `--ga-ecommerce` | No | `false` | Enable GA e-commerce tracking |
| `--utm-campaign` | No | — | Custom `utm_campaign` value |
| `--utm-content` | No | — | Custom `utm_content` value |
| `--wait` | No | `false` | Poll campaign status until sending completes |

## Workflow

1. **Check balance** — verifies sufficient email credits before sending.
2. **Submit campaign** — builds the request from parameters and submits it.
3. **(if `--wait`)** **Poll status** — spinner polls every 5 s (timeout 600 s).
4. **(if `--wait`)** **Return metrics** — once COMPLETE/SENT, fetches final
   performance metrics (opens, clicks, bounces).

## Examples

```bash
# Send immediately and wait for completion
nlm helper campaign-send \
  --name "March Newsletter" \
  --lists GRP-001,GRP-002 \
  --subject "March deals inside" \
  --from-name "ACME Corp" \
  --from-address news@acme.com \
  --html-file ./march-newsletter.html \
  --wait

# Schedule for later (no waiting needed)
nlm x campaign-send \
  --name "Holiday Sale" \
  --lists GRP-001 \
  --subject "Holiday specials for ${FIRST_NAME}" \
  --from-name "ACME" \
  --from-address promo@acme.com \
  --html-file sale.html \
  --schedule scheduled \
  --schedule-date "2025-12-20T09:00:00" \
  --schedule-timezone 8 \
  --ga --utm-campaign "holiday-2025"

# Dry-run preview (no API call)
nlm helper campaign-send \
  --name "Test" --lists GRP-001 --subject "Hi" \
  --from-name "Test" --from-address test@example.com \
  --html "<p>Hello</p>" --dry-run
```

## Tips

- Use `--dry-run` to preview the request payload without sending.
- The `--html` and `--html-file` flags are mutually exclusive.
- EDM variable syntax is `${FIELD_NAME}`. Using `{{...}}` triggers a warning.
- Without `--wait`, the command returns immediately after submit with the
  campaign SN. You can check status later with `nlm edm campaign status`.
- Balance check failure (zero credits) returns exit code 2 (Validation).
"#;

const IMPORT_AND_WAIT_BODY: &str = r#"# nlm helper import-and-wait

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
"#;

const REPORT_DOWNLOAD_BODY: &str = r#"# nlm helper report-download

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
"#;

const DOMAIN_SETUP_BODY: &str = r#"# nlm helper domain-setup

> **Prerequisite skills:** `nlm-shared`, `nlm-sn`, `nlm-sn-domain`

Set up a sender domain with DNS verification records and optional automatic
verification after a waiting period.

## Usage

```bash
nlm helper domain-setup --domain <DOMAIN> [--auto-verify-after <SECONDS>]
```

Alias: `nlm x domain-setup ...`

## Parameters

| Parameter | Required | Default | Description |
|-----------|----------|---------|-------------|
| `--domain` | Yes | — | Domain to set up (e.g. `mail.example.com`) |
| `--auto-verify-after` | No | — | Seconds to wait before attempting automatic DNS verification |

## Workflow

1. **Create domain** — registers the domain via the Surenotify API.
2. **Display DNS records** — prints the required TXT and CNAME records to
   stderr for you to configure in your DNS provider.
3. **(if `--auto-verify-after`)** **Wait** — spinner counts down the specified
   seconds for DNS propagation.
4. **(if `--auto-verify-after`)** **Verify** — triggers domain verification
   and returns the result.

## Examples

```bash
# Manual verification (shows DNS records, then exits)
nlm helper domain-setup --domain mail.example.com

# Auto-verify after 5 minutes (300 s) of DNS propagation
nlm x domain-setup --domain mail.example.com --auto-verify-after 300

# Auto-verify after 1 minute (quick check)
nlm helper domain-setup --domain mail.example.com --auto-verify-after 60
```

## Tips

- Without `--auto-verify-after`, the command returns the DNS records and exits.
  You can verify later with `nlm sn domain verify --domain <DOMAIN>`.
- DNS propagation typically takes 1-5 minutes but can take up to 48 hours
  depending on your DNS provider and TTL settings.
- The DNS records printed to stderr include record type (TXT/CNAME), name,
  and value. Configure all of them before verifying.
- This command uses the **Surenotify API** (not EDM), so it requires
  `NL_SN_API_KEY` to be configured.
"#;

// ── Skill definitions ───────────────────────────────────────────

pub fn skills() -> Vec<SkillDefinition> {
    vec![
        SkillDefinition {
            name: "nlm-helper-campaign-send".to_string(),
            version: "1.0.0".to_string(),
            description: "nlm helper: Submit a campaign and optionally wait for completion \
                 — combines submit + status polling."
                .to_string(),
            category: SkillCategory::Helper,
            domain: Some("campaign".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-shared".to_string(),
                "nlm-edm".to_string(),
                "nlm-edm-campaign".to_string(),
            ],
            body: CAMPAIGN_SEND_BODY.to_string(),
        },
        SkillDefinition {
            name: "nlm-helper-import-and-wait".to_string(),
            version: "1.0.0".to_string(),
            description:
                "nlm helper: Import contacts from a file and poll until the import completes."
                    .to_string(),
            category: SkillCategory::Helper,
            domain: Some("contacts".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-shared".to_string(),
                "nlm-edm".to_string(),
                "nlm-edm-contacts".to_string(),
            ],
            body: IMPORT_AND_WAIT_BODY.to_string(),
        },
        SkillDefinition {
            name: "nlm-helper-report-download".to_string(),
            version: "1.0.0".to_string(),
            description: "nlm helper: Export a campaign report and download the file \
                 — combines export + poll + download."
                .to_string(),
            category: SkillCategory::Helper,
            domain: Some("report".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-shared".to_string(),
                "nlm-edm".to_string(),
                "nlm-edm-report".to_string(),
            ],
            body: REPORT_DOWNLOAD_BODY.to_string(),
        },
        SkillDefinition {
            name: "nlm-helper-domain-setup".to_string(),
            version: "1.0.0".to_string(),
            description:
                "nlm helper: Set up a sender domain with DNS records and optional auto-verify."
                    .to_string(),
            category: SkillCategory::Helper,
            domain: Some("domain".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-shared".to_string(),
                "nlm-sn".to_string(),
                "nlm-sn-domain".to_string(),
            ],
            body: DOMAIN_SETUP_BODY.to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_four_skills() {
        let s = skills();
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn all_names_start_with_nlm_helper() {
        for skill in skills() {
            assert!(
                skill.name.starts_with("nlm-helper-"),
                "Skill name '{}' should start with 'nlm-helper-'",
                skill.name
            );
        }
    }

    #[test]
    fn all_are_helper_category() {
        for skill in skills() {
            assert!(
                matches!(skill.category, SkillCategory::Helper),
                "Skill '{}' should be Helper category",
                skill.name
            );
        }
    }

    #[test]
    fn all_have_domain() {
        for skill in skills() {
            assert!(
                skill.domain.is_some(),
                "Skill '{}' should have a domain",
                skill.name
            );
        }
    }

    #[test]
    fn all_require_nlm_bin() {
        for skill in skills() {
            assert!(
                skill.requires_bins.contains(&"nlm".to_string()),
                "Skill '{}' should require nlm binary",
                skill.name
            );
        }
    }

    #[test]
    fn campaign_send_requires_correct_skills() {
        let s = skills();
        let cs = s
            .iter()
            .find(|s| s.name == "nlm-helper-campaign-send")
            .unwrap();
        assert_eq!(
            cs.requires_skills,
            vec!["nlm-shared", "nlm-edm", "nlm-edm-campaign"]
        );
    }

    #[test]
    fn domain_setup_requires_sn_skills() {
        let s = skills();
        let ds = s
            .iter()
            .find(|s| s.name == "nlm-helper-domain-setup")
            .unwrap();
        assert_eq!(
            ds.requires_skills,
            vec!["nlm-shared", "nlm-sn", "nlm-sn-domain"]
        );
    }

    #[test]
    fn bodies_contain_alias_reference() {
        for skill in skills() {
            assert!(
                skill.body.contains("nlm x"),
                "Skill '{}' body should mention the 'nlm x' alias",
                skill.name
            );
        }
    }
}
