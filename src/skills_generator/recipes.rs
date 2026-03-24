//! Recipe skills: multi-step workflows with real nlm commands.

use super::{SkillCategory, SkillDefinition};

pub fn skills() -> Vec<SkillDefinition> {
    vec![
        SkillDefinition {
            name: "recipe-weekly-newsletter".to_string(),
            version: "1.0.0".to_string(),
            description: "Send a weekly newsletter: prepare content, submit campaign, and review results.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("campaign".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-campaign".to_string(),
                "nlm-edm-contacts".to_string(),
                "nlm-edm-report".to_string(),
            ],
            body: RECIPE_WEEKLY_NEWSLETTER.to_string(),
        },
        SkillDefinition {
            name: "recipe-ab-test-subject".to_string(),
            version: "1.0.0".to_string(),
            description: "A/B test two subject lines to optimize open rates.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("ab-test".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-ab-test".to_string(),
                "nlm-edm-report".to_string(),
            ],
            body: RECIPE_AB_TEST_SUBJECT.to_string(),
        },
        SkillDefinition {
            name: "recipe-import-and-send".to_string(),
            version: "1.0.0".to_string(),
            description: "Import a contact list from CSV, wait for completion, then send a campaign to the new list.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("contacts".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-contacts".to_string(),
                "nlm-edm-campaign".to_string(),
            ],
            body: RECIPE_IMPORT_AND_SEND.to_string(),
        },
        SkillDefinition {
            name: "recipe-campaign-performance-review".to_string(),
            version: "1.0.0".to_string(),
            description: "Review campaign performance: list recent campaigns, compare metrics, and export detailed report.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("report".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-report".to_string(),
                "nlm-edm-campaign".to_string(),
            ],
            body: RECIPE_CAMPAIGN_PERFORMANCE_REVIEW.to_string(),
        },
        SkillDefinition {
            name: "recipe-transactional-email-setup".to_string(),
            version: "1.0.0".to_string(),
            description: "Set up transactional email: verify domain, configure webhooks, and send a test email.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("email".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-sn-email".to_string(),
                "nlm-sn-domain".to_string(),
                "nlm-sn-webhook".to_string(),
            ],
            body: RECIPE_TRANSACTIONAL_EMAIL_SETUP.to_string(),
        },
        SkillDefinition {
            name: "recipe-sms-notification".to_string(),
            version: "1.0.0".to_string(),
            description: "Send SMS notifications: configure webhooks and send messages with delivery tracking.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("sms".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-sn-sms".to_string(),
                "nlm-sn-sms-webhook".to_string(),
            ],
            body: RECIPE_SMS_NOTIFICATION.to_string(),
        },
        SkillDefinition {
            name: "recipe-domain-migration".to_string(),
            version: "1.0.0".to_string(),
            description: "Migrate sender domain: set up new domain, verify DNS, then remove old domain.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("domain".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-sn-domain".to_string(),
            ],
            body: RECIPE_DOMAIN_MIGRATION.to_string(),
        },
        SkillDefinition {
            name: "recipe-mcp-tool-exploration".to_string(),
            version: "1.0.0".to_string(),
            description: "Discover and use MCP tools: list available tools, understand schemas, and invoke them.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("mcp".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-mcp".to_string(),
            ],
            body: RECIPE_MCP_TOOL_EXPLORATION.to_string(),
        },
        SkillDefinition {
            name: "recipe-multi-profile-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: "Manage staging and production environments with config profiles.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("config".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-config".to_string(),
            ],
            body: RECIPE_MULTI_PROFILE_WORKFLOW.to_string(),
        },
        SkillDefinition {
            name: "recipe-contact-cleanup".to_string(),
            version: "1.0.0".to_string(),
            description: "Clean up contact lists: identify low-engagement contacts and remove bounced addresses.".to_string(),
            category: SkillCategory::Recipe,
            domain: Some("contacts".to_string()),
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-contacts".to_string(),
                "nlm-edm-report".to_string(),
            ],
            body: RECIPE_CONTACT_CLEANUP.to_string(),
        },
    ]
}

// ---------------------------------------------------------------------------
// Recipe body constants
// ---------------------------------------------------------------------------

const RECIPE_WEEKLY_NEWSLETTER: &str = r#"# Recipe: Weekly Newsletter

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
  hands-off weekly sends."#;

const RECIPE_AB_TEST_SUBJECT: &str = r#"# Recipe: A/B Test Subject Lines

Run a subject-line A/B test to find which version drives higher open rates,
then review results.

## Prerequisites

- EDM API key configured
- At least one contact list with enough subscribers for a meaningful test
- HTML content file for the campaign body
- Verified sender address

---

## Steps

### Step 1 — Submit the A/B test

Create a campaign with two subject-line variants. The API splits the test
group evenly and sends each variant to half.

```bash
nlm edm ab-test submit \
  --name 'Subject Test' \
  --lists L1 \
  --subject-a 'Version A' \
  --subject-b 'Version B' \
  --from-name F \
  --from-address A \
  --html H
```

The command outputs serial numbers for each variant.

### Step 2 — Wait and check the report

Allow enough time for recipients to open (typically 1-4 hours), then pull
the metrics.

```bash
nlm edm report metrics --sns CAM_SN
```

Look at `open_rate` and `click_rate` for each variant.

### Step 3 — Compare results

Side-by-side comparison of the two variants.

```bash
nlm edm campaign compare --sns SN_A SN_B
```

This outputs a diff-style table showing open rate, click rate, bounce rate,
and unsubscribe rate for each variant.

---

## Tips

- **Sample size matters:** A list smaller than 1,000 may not produce
  statistically significant results. Aim for at least 500 per variant.
- **Test one variable:** Keep the body HTML identical; only change the subject
  line. Testing multiple variables at once makes it impossible to attribute
  differences.
- **Timing:** Send both variants simultaneously so time-of-day does not skew
  results.
- **Follow up:** Once you identify the winner, use that subject line for the
  full campaign send to the remaining list."#;

const RECIPE_IMPORT_AND_SEND: &str = r#"# Recipe: Import Contacts and Send

Import a contact list from a CSV file, wait for the import to finish, then
immediately send a campaign to the newly created list.

## Prerequisites

- EDM API key configured
- A CSV file with at least an `email` column (e.g., `contacts.csv`)
- HTML content file for the campaign
- Verified sender address

---

## Steps

### Step 1 — Create a new contact group

```bash
nlm edm contacts create-group --name 'March Promo'
```

Note the new group serial number (`NEW_SN`) from the output.

### Step 2 — Import contacts and wait

The `import-and-wait` helper uploads the CSV file and polls until the import
job completes.

```bash
nlm helper import-and-wait --list-sn NEW_SN --file contacts.csv
```

The helper reports progress (records processed, duplicates, errors) and exits
once the import is fully complete.

### Step 3 — Send the campaign

```bash
nlm helper campaign-send \
  --name 'March Promo' \
  --lists NEW_SN \
  --subject 'Special Offer' \
  --from-name 'ACME Sales' \
  --from-address sales@acme.com \
  --html-file promo.html \
  --wait
```

---

## Tips

- **CSV format:** The first row must be headers. At minimum, include `email`.
  Optional columns: `name`, `phone`, custom fields defined in your account.
- **Deduplication:** The API automatically deduplicates against existing
  contacts in the group. Duplicate rows in the CSV are counted but not
  imported twice.
- **Large files:** For files over 100,000 rows, the import may take several
  minutes. The `--wait` flag on `import-and-wait` handles this automatically.
- **Dry-run the send:** Add `--dry-run` to the `campaign-send` step to verify
  the request payload before committing."#;

const RECIPE_CAMPAIGN_PERFORMANCE_REVIEW: &str = r#"# Recipe: Campaign Performance Review

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
  automated threshold checks (e.g., alert if bounce rate exceeds 5%)."#;

const RECIPE_TRANSACTIONAL_EMAIL_SETUP: &str = r#"# Recipe: Transactional Email Setup

Set up transactional email from scratch — verify a sender domain, configure
delivery webhooks, send a test email, and confirm delivery.

## Prerequisites

- Surenotify API key configured (`nlm config set sn_api_key "..."`)
- Access to DNS management for your sending domain
- A webhook endpoint URL (e.g., `https://hooks.example.com/delivered`)

---

## Steps

### Step 1 — Set up and verify the sender domain

The `domain-setup` helper creates the domain, displays required DNS records,
and optionally waits for DNS propagation before verifying.

```bash
nlm helper domain-setup \
  --domain mail.example.com \
  --auto-verify-after 60
```

The `--auto-verify-after 60` flag waits 60 seconds then attempts verification.
Add the displayed CNAME and TXT records to your DNS provider while waiting.

### Step 2 — Create delivery webhooks

Register a webhook endpoint to receive real-time delivery events.

```bash
nlm sn webhook create \
  --event-type delivered \
  --url https://hooks.example.com/delivered
```

Repeat for other event types as needed: `bounced`, `opened`, `clicked`,
`complained`, `unsubscribed`.

### Step 3 — Send a test email

```bash
nlm sn email send \
  --subject 'Test' \
  --from-address noreply@mail.example.com \
  --html '<p>Test email</p>' \
  --to test@example.com
```

### Step 4 — Check delivery events

Verify the test email was delivered and the webhook fired.

```bash
nlm sn email events --email test@example.com
```

Look for a `delivered` event. If you see `bounced` instead, check your DNS
records and sender domain verification status.

---

## Tips

- **DNS propagation:** CNAME and TXT records can take up to 48 hours to
  propagate, though most providers update within minutes. Use
  `dig CNAME mail.example.com` to check locally.
- **Multiple event types:** Create separate webhooks for `delivered`,
  `bounced`, and `complained` to route events to different handlers.
- **Webhook security:** Use HTTPS endpoints and validate the webhook
  signature header to prevent spoofing.
- **Variables:** Use `{{variable_name}}` syntax in Surenotify templates.
  Remember: this is different from EDM's `${FIELD_NAME}` syntax."#;

const RECIPE_SMS_NOTIFICATION: &str = r#"# Recipe: SMS Notification

Send SMS notifications with delivery tracking — configure a webhook for
delivery receipts, send a message, and verify delivery.

## Prerequisites

- Surenotify API key configured (`nlm config set sn_api_key "..."`)
- SMS credits in your Surenotify account
- A webhook endpoint URL for delivery receipts

---

## Steps

### Step 1 — Create a delivery webhook

Register a webhook to receive SMS delivery status updates.

```bash
nlm sn sms-webhook create \
  --event-type delivered \
  --url https://hooks.example.com/sms
```

### Step 2 — Send an SMS message

```bash
nlm sn sms send \
  --content 'ACME: Your order is shipped. Track at https://acme.com/track/123' \
  --phone 0912345678 \
  --country-code 886
```

The command outputs a message ID for tracking.

### Step 3 — Check delivery events

```bash
nlm sn sms events
```

Look for the message ID from Step 2 and confirm the status is `delivered`.

---

## Tips

- **Character limits:** Standard SMS messages are limited to 160 characters
  (70 for messages containing non-ASCII characters like Chinese). Messages
  exceeding this limit are split into multiple segments and billed accordingly.
- **Country codes:** Use the numeric country code without the `+` prefix.
  Taiwan is `886`, US is `1`, Japan is `81`.
- **URL shortening:** Long URLs consume valuable character space. Consider
  using a URL shortener to keep messages concise.
- **Opt-out compliance:** Include opt-out instructions in marketing SMS
  (e.g., "Reply STOP to unsubscribe"). Transactional messages (order
  confirmations, OTPs) are typically exempt."#;

const RECIPE_DOMAIN_MIGRATION: &str = r#"# Recipe: Domain Migration

Migrate from an old sender domain to a new one — create the new domain,
verify DNS records, confirm it works, then remove the old domain.

## Prerequisites

- Surenotify API key configured
- Access to DNS management for the new domain
- Knowledge of the old domain to be removed

---

## Steps

### Step 1 — Create the new domain

```bash
nlm sn domain create --domain new.example.com
```

The output displays the required DNS records (CNAME, TXT) that must be added
to your DNS provider.

### Step 2 — Configure DNS records

Add the displayed records to your DNS provider:

- **CNAME record:** Points the sending subdomain to Surenotify's mail servers.
- **TXT record:** SPF/DKIM verification for email authentication.

Wait for DNS propagation (typically a few minutes, up to 48 hours).

### Step 3 — Verify the new domain

```bash
nlm sn domain verify --domain new.example.com
```

If verification fails, double-check the DNS records with `dig` and retry.
The API returns specific error messages indicating which records are missing
or incorrect.

### Step 4 — Remove the old domain

Once the new domain is verified and tested, remove the old one.

```bash
nlm sn domain remove --domain old.example.com
```

---

## Tips

- **Test before removing:** Send a test email via the new domain
  (`nlm sn email send --from-address noreply@new.example.com ...`) and
  confirm delivery before removing the old domain.
- **Gradual migration:** If you have high volume, consider running both
  domains in parallel for a transition period to catch any issues.
- **DNS TTL:** Lower the TTL on old DNS records before migration to speed up
  the cutover. Restore normal TTL values after the migration is complete.
- **Update templates:** After migration, update all email templates and
  application code to use the new `from-address` domain."#;

const RECIPE_MCP_TOOL_EXPLORATION: &str = r#"# Recipe: MCP Tool Exploration

Discover and use MCP (Model Context Protocol) tools — list what is available,
inspect tool schemas, and invoke a tool with parameters.

## Prerequisites

- MCP server base URL configured (`nlm config set mcp_url "https://..."`; `nlm` uses the `/mcp` endpoint)
- The MCP server must be running and accessible

---

## Steps

### Step 1 — List available tools

Discover all tools exposed by the MCP server.

```bash
nlm mcp tools
```

This returns a JSON array of tool definitions, each with a `name`,
`description`, and `inputSchema`.

### Step 2 — Inspect a specific tool

Filter the tool list to examine one tool's schema in detail.

```bash
nlm mcp tools | jq '.[] | select(.name == "analyze_campaign")'
```

Review the `inputSchema` to understand required and optional parameters.

### Step 3 — Call a tool

Invoke the tool with a JSON argument payload.

```bash
nlm mcp call analyze_campaign --json '{"campaign_sn": "CAM12345"}'
```

The command sends a JSON-RPC 2.0 request to the MCP server and returns the
tool's response.

---

## Tips

- **Schema validation:** The CLI validates `--json` against the tool's
  `inputSchema` before sending the request. Missing required fields produce
  a validation error (exit code 2).
- **Output formatting:** MCP responses are JSON by default. Use `--format table`
  for a readable summary or pipe to `jq` for field extraction.
- **Tool discovery for agents:** AI agents can call `nlm mcp tools` to
  dynamically discover available capabilities — this is the primary use case
  for MCP integration.
- **Debugging:** Use `-vv` to see the full JSON-RPC request and response
  for troubleshooting."#;

const RECIPE_MULTI_PROFILE_WORKFLOW: &str = r#"# Recipe: Multi-Profile Workflow

Manage staging and production environments using config profiles — create
profiles, set per-environment API keys, and switch between them.

## Prerequisites

- `nlm` installed
- API keys for each environment (staging, production)

---

## Steps

### Step 1 — Initialize default config

Run the interactive setup wizard to configure your default (production)
profile.

```bash
nlm config init
```

Enter your production API keys when prompted.

### Step 2 — Create a staging profile

```bash
nlm config profile create staging
```

### Step 3 — Set staging API keys

```bash
nlm config set edm_api_key "staging-key" --profile staging
nlm config set sn_api_key "staging-sn-key" --profile staging
```

### Step 4 — Test the staging profile

Verify the staging keys work by checking the account balance.

```bash
nlm edm account balance --profile staging
```

### Step 5 — Use the production profile

Switch back to production (the default profile) for live operations.

```bash
nlm edm account balance --profile default
```

Or simply omit the `--profile` flag — `default` is used automatically.

---

## Tips

- **Environment variables override profiles:** If `NL_EDM_API_KEY` is set, it
  takes precedence over any profile. Unset it when switching profiles
  interactively.
- **CI/CD pattern:** In CI pipelines, use environment variables instead of
  profiles: `NL_EDM_API_KEY=... nlm edm account balance`. This avoids
  persisting secrets to disk.
- **List all profiles:** Use `nlm config profile list` to see available
  profiles and which is currently active.
- **Per-profile format:** Set `default_format` per profile — e.g., `table`
  for staging (human review) and `json` for production (scripting)."#;

const RECIPE_CONTACT_CLEANUP: &str = r#"# Recipe: Contact Cleanup

Clean up contact lists by identifying low-engagement contacts, reviewing
bounced addresses from campaign reports, and removing invalid entries.

## Prerequisites

- EDM API key configured
- At least one contact list with historical campaign data
- Recent campaign reports available for bounce analysis

---

## Steps

### Step 1 — Check top lists

Review your largest and most active contact lists.

```bash
nlm edm contacts top-lists --format table
```

Identify lists that may need cleanup based on size and last-send date.

### Step 2 — Review campaign bounces

Pull bounce data from recent campaign reports to identify invalid addresses.

```bash
nlm edm report metrics --sns CAM1 --format table
```

Check the `bounce_rate` field. If it exceeds 2-3%, the list needs cleanup.
For detailed bounce data, export the report:

```bash
nlm helper report-download --sn CAM1 --output bounces.csv
```

Filter the CSV for rows with `status = bounced` to get the list of addresses
to remove.

### Step 3 — Remove bounced contacts

Remove a specific bounced address from a list.

```bash
nlm edm contacts remove \
  --list-sn L1 \
  --field email \
  --op eq \
  --value bounced@example.com
```

For bulk removal, script this step by looping over the bounced addresses
extracted from the CSV:

```bash
while IFS=, read -r email _rest; do
  nlm edm contacts remove --list-sn L1 --field email --op eq --value "$email"
done < bounced-emails.txt
```

---

## Tips

- **Bounce types:** Hard bounces (invalid address, domain does not exist)
  should be removed immediately. Soft bounces (mailbox full, server
  temporarily unavailable) may resolve on their own — retry before removing.
- **Regular cadence:** Run this cleanup monthly to maintain list hygiene and
  protect your sender reputation.
- **Rate limits:** The EDM API allows 2 requests per second. The bulk removal
  loop above naturally stays within this limit, but for very large lists
  consider adding a small delay.
- **Backup first:** Export the full contact list before removing entries so
  you can restore if needed:
  `nlm edm contacts list --list-sn L1 --format csv > backup.csv`"#;
