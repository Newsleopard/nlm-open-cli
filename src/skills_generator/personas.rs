//! Persona skills: role-based skill bundles for common user archetypes.

use super::{SkillCategory, SkillDefinition};

pub fn skills() -> Vec<SkillDefinition> {
    vec![
        SkillDefinition {
            name: "persona-email-marketer".to_string(),
            version: "1.0.0".to_string(),
            description: "Email marketer: Plan campaigns, manage audiences, test content, and analyze performance.".to_string(),
            category: SkillCategory::Persona,
            domain: None,
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-campaign".to_string(),
                "nlm-edm-contacts".to_string(),
                "nlm-edm-report".to_string(),
                "nlm-edm-template".to_string(),
                "nlm-edm-ab-test".to_string(),
            ],
            body: PERSONA_EMAIL_MARKETER.to_string(),
        },
        SkillDefinition {
            name: "persona-devops-engineer".to_string(),
            version: "1.0.0".to_string(),
            description: "DevOps engineer: Configure transactional messaging infrastructure, domains, and webhooks.".to_string(),
            category: SkillCategory::Persona,
            domain: None,
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-sn-email".to_string(),
                "nlm-sn-domain".to_string(),
                "nlm-sn-webhook".to_string(),
                "nlm-sn-sms-webhook".to_string(),
                "nlm-config".to_string(),
            ],
            body: PERSONA_DEVOPS_ENGINEER.to_string(),
        },
        SkillDefinition {
            name: "persona-developer".to_string(),
            version: "1.0.0".to_string(),
            description: "Developer: Integrate transactional email and SMS into applications using the nlm CLI.".to_string(),
            category: SkillCategory::Persona,
            domain: None,
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-sn-email".to_string(),
                "nlm-sn-sms".to_string(),
                "nlm-mcp".to_string(),
                "nlm-config".to_string(),
            ],
            body: PERSONA_DEVELOPER.to_string(),
        },
        SkillDefinition {
            name: "persona-customer-success".to_string(),
            version: "1.0.0".to_string(),
            description: "Customer success: Monitor campaign health, track engagement, and manage subscriber lists.".to_string(),
            category: SkillCategory::Persona,
            domain: None,
            requires_bins: vec!["nlm".to_string()],
            requires_skills: vec![
                "nlm-edm-campaign".to_string(),
                "nlm-edm-report".to_string(),
                "nlm-edm-contacts".to_string(),
            ],
            body: PERSONA_CUSTOMER_SUCCESS.to_string(),
        },
    ]
}

const PERSONA_EMAIL_MARKETER: &str = r#"# Persona — Email Marketer

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
- Use `--dry-run` liberally — it costs nothing and prevents accidental sends."#;

const PERSONA_DEVOPS_ENGINEER: &str = r#"# Persona — DevOps Engineer

Role-based skill bundle for DevOps engineers who configure transactional messaging
infrastructure, manage sender domains, set up webhooks, and maintain multi-environment
configurations.

## Prerequisite Skills

This persona depends on the following skills for full command reference:

- **nlm-sn-email** — Send transactional emails and query delivery events
- **nlm-sn-domain** — Configure and verify sender domains
- **nlm-sn-webhook** — Manage email delivery webhooks
- **nlm-sn-sms-webhook** — Manage SMS delivery webhooks
- **nlm-config** — Profile management, credentials, and environment setup

## Relevant Workflows

- **recipe-transactional-email-setup** — End-to-end transactional email configuration
- **recipe-domain-migration** — Migrate sender domains between providers
- **recipe-multi-profile-workflow** — Manage staging/production profiles

## Instructions

1. **Set up profiles for each environment** to avoid mixing staging and production:

   ```bash
   nlm config profile create staging
   nlm config set edm_api_key "staging-key" --profile staging
   nlm config set sn_api_key "staging-key" --profile staging

   nlm config profile create production
   nlm config set edm_api_key "prod-key" --profile production
   nlm config set sn_api_key "prod-key" --profile production
   ```

2. **Configure sender domains** with automatic DNS verification:

   ```bash
   nlm helper domain-setup --domain mail.example.com --auto-verify-after 60
   ```

   The `--auto-verify-after` flag polls DNS propagation and verifies after the
   specified number of seconds.

3. **Set up delivery webhooks** to monitor email and SMS delivery events in your
   monitoring infrastructure:

   ```bash
   nlm sn webhook create --url https://hooks.example.com/delivery --events delivered,bounced,complained
   nlm sn sms-webhook create --url https://hooks.example.com/sms --events delivered,failed
   ```

4. **Use `--dry-run` to validate API calls** before executing in production:

   ```bash
   nlm sn email send --to test@example.com --subject "Infra test" --dry-run --profile production
   ```

5. **Monitor delivery events** for operational health:

   ```bash
   nlm sn email events --format table
   nlm sn sms events --format table
   ```

6. **Use `--format json` for pipeline integration** — JSON output auto-compacts when
   stdout is piped, making it easy to feed into `jq`, monitoring scripts, or log
   aggregators:

   ```bash
   nlm sn email events --format json | jq '.[] | select(.event_type == "bounced")'
   ```

## Tips

- Always test domain and webhook configuration in the staging profile before applying
  to production.
- Webhook endpoints should return 2xx quickly — the Surenotify API may retry on
  timeouts.
- Use environment variables (`NL_SN_API_KEY`, `NL_EDM_API_KEY`) in CI/CD pipelines
  instead of config files for better secret management.
- Run `nlm config list --profile production` periodically to verify configuration
  (API keys are always masked in output)."#;

const PERSONA_DEVELOPER: &str = r#"# Persona — Developer

Role-based skill bundle for developers integrating transactional email and SMS into
applications using the `nlm` CLI for prototyping, testing, and scripting.

## Prerequisite Skills

This persona depends on the following skills for full command reference:

- **nlm-sn-email** — Send transactional emails and query delivery events
- **nlm-sn-sms** — Send transactional SMS messages and query delivery events
- **nlm-mcp** — MCP tool discovery and invocation for AI-powered features
- **nlm-config** — Profile management and environment setup

## Relevant Workflows

- **recipe-transactional-email-setup** — End-to-end transactional email configuration
- **recipe-mcp-tool-exploration** — Discover and invoke MCP tools

## Instructions

1. **Use MCP tools for AI-powered features** — discover available tools and invoke
   them programmatically:

   ```bash
   nlm mcp tools
   nlm mcp call tool-name '{"param": "value"}'
   ```

2. **Send transactional emails with template variables** using Surenotify's
   `{{variable}}` syntax:

   ```bash
   nlm sn email send \
     --to user@example.com \
     --subject 'Order {{order_id}}' \
     --html '<p>Hi {{name}}, your order is confirmed.</p>'
   ```

3. **Pipe output to `jq` for scripting** — JSON auto-compacts when stdout is piped:

   ```bash
   nlm sn email events | jq '.[] | .event_type'
   nlm sn sms events | jq '.[] | select(.status == "delivered")'
   ```

4. **Use exit codes in scripts** for robust error handling:

   ```bash
   nlm sn email send --to user@example.com --subject "Test" --html "<p>Hi</p>"
   case $? in
     0) echo "Sent successfully" ;;
     1) echo "API rejected the request" ;;
     2) echo "Invalid parameters" ;;
     3) echo "Check your API key" ;;
     4) echo "Network issue or rate limited" ;;
     5) echo "File I/O problem" ;;
   esac
   ```

5. **Use `--dry-run` for testing** API calls without side effects — ideal for
   development and CI:

   ```bash
   nlm sn email send --to test@example.com --subject "CI test" --dry-run
   ```

6. **JSON output auto-compacts when stdout is piped**, so you can chain commands
   without worrying about pretty-print formatting breaking parsers.

## Tips

- Use `NL_SN_API_KEY` and `NL_EDM_API_KEY` environment variables in CI pipelines
  for clean secret management.
- The `--dry-run` flag outputs the HTTP request that *would* be sent — useful for
  debugging template variable interpolation.
- MCP tools extend the CLI with AI-powered capabilities; run `nlm mcp tools` to see
  what is available in your configured MCP server.
- Remember: Surenotify uses `{{variable}}` syntax, EDM uses `${VARIABLE}`. The CLI
  warns if you mix them up."#;

const PERSONA_CUSTOMER_SUCCESS: &str = r#"# Persona — Customer Success

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
  hygiene issues that need attention."#;
