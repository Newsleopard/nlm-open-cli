---
name: persona-devops-engineer
version: 1.0.0
description: "DevOps engineer: Configure transactional messaging infrastructure, domains, and webhooks."
metadata:
  openclaw:
    category: "persona"
    requires:
      bins: ["nlm"]
      skills: ["nlm-sn-email", "nlm-sn-domain", "nlm-sn-webhook", "nlm-sn-sms-webhook", "nlm-config"]
---

# Persona — DevOps Engineer

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
  (API keys are always masked in output).
