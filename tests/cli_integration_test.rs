//! CLI end-to-end integration tests for the `nlm` binary.
//!
//! These tests exercise the full command-line interface via `assert_cmd`, verifying:
//! - Help output for all top-level and nested subcommands
//! - Version string
//! - Exit codes for authentication failures, dry-run, and invalid arguments
//! - Variable syntax warnings
//! - Output format flags
//! - Config commands (no API key needed)
//! - Conflicting argument detection
//! - Quiet mode suppression
//!
//! All tests are self-contained and isolated: environment variables that could
//! leak configuration are removed with `.env_remove()`, and API keys are injected
//! only via `.env()` where needed.

use assert_cmd::Command;
use predicates::prelude::*;

/// Construct a `Command` for the `nlm` binary from the cargo build.
fn nl() -> Command {
    let mut cmd = Command::cargo_bin("nlm").unwrap();
    // Ensure no stale config or env vars leak into the test environment.
    cmd.env_remove("NL_EDM_API_KEY");
    cmd.env_remove("NL_SN_API_KEY");
    cmd.env_remove("NL_FORMAT");
    cmd.env_remove("NL_PROFILE");
    cmd.env_remove("RUST_LOG");
    cmd
}

// ══════════════════════════════════════════════════════════════════════════════
// 1. Help output
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn help_top_level() {
    nl().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Newsleopard email marketing (EDM) API",
        ));
}

#[test]
fn help_edm() {
    nl().args(["edm", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Newsleopard EDM API"));
}

#[test]
fn help_sn() {
    nl().args(["sn", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Surenotify transactional messaging API",
        ));
}

#[test]
fn help_config() {
    nl().args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration management"));
}

#[test]
fn help_helper() {
    nl().args(["helper", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "High-level orchestration workflows",
        ));
}

#[test]
fn help_x_alias() {
    // `nlm x` is an alias for `nlm helper`
    nl().args(["x", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "High-level orchestration workflows",
        ));
}

// ══════════════════════════════════════════════════════════════════════════════
// 2. Version
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn version_output() {
    nl().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

// ══════════════════════════════════════════════════════════════════════════════
// 3. Subcommand help
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn help_edm_contacts() {
    nl().args(["edm", "contacts", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("create-group")
                .and(predicate::str::contains("list-groups"))
                .and(predicate::str::contains("import-file")),
        );
}

#[test]
fn help_edm_campaign() {
    nl().args(["edm", "campaign", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("submit")
                .and(predicate::str::contains("submit-once"))
                .and(predicate::str::contains("delete"))
                .and(predicate::str::contains("pause"))
                .and(predicate::str::contains("status")),
        );
}

#[test]
fn help_edm_ab_test() {
    nl().args(["edm", "ab-test", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("submit").and(predicate::str::contains("submit-once")));
}

#[test]
fn help_edm_report() {
    nl().args(["edm", "report", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("list")
                .and(predicate::str::contains("metrics"))
                .and(predicate::str::contains("export"))
                .and(predicate::str::contains("download-link")),
        );
}

#[test]
fn help_edm_template() {
    nl().args(["edm", "template", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list").and(predicate::str::contains("get")));
}

#[test]
fn help_edm_automation() {
    nl().args(["edm", "automation", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trigger"));
}

#[test]
fn help_edm_account() {
    nl().args(["edm", "account", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("balance"));
}

#[test]
fn help_sn_email() {
    nl().args(["sn", "email", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("send").and(predicate::str::contains("events")));
}

#[test]
fn help_sn_sms() {
    nl().args(["sn", "sms", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("send")
                .and(predicate::str::contains("events"))
                .and(predicate::str::contains("exclusive-number")),
        );
}

#[test]
fn help_sn_webhook() {
    nl().args(["sn", "webhook", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("create")
                .and(predicate::str::contains("list"))
                .and(predicate::str::contains("delete")),
        );
}

#[test]
fn help_sn_sms_webhook() {
    nl().args(["sn", "sms-webhook", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("create")
                .and(predicate::str::contains("list"))
                .and(predicate::str::contains("delete")),
        );
}

#[test]
fn help_sn_domain() {
    nl().args(["sn", "domain", "--help"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("create")
                .and(predicate::str::contains("verify"))
                .and(predicate::str::contains("remove")),
        );
}

// ══════════════════════════════════════════════════════════════════════════════
// 4. Missing API key (exit code 3)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn missing_edm_api_key_balance() {
    nl().args(["edm", "account", "balance"])
        .assert()
        .code(3)
        .stderr(
            predicate::str::contains("Authentication error")
                .and(predicate::str::contains("EDM API key not configured")),
        );
}

#[test]
fn missing_sn_api_key_email_send() {
    nl().args([
        "sn",
        "email",
        "send",
        "--subject",
        "test",
        "--from-address",
        "a@b.com",
        "--html",
        "<p>x</p>",
        "--to",
        "a@b.com",
    ])
    .assert()
    .code(3)
    .stderr(
        predicate::str::contains("Authentication error").and(predicate::str::contains(
            "Surenotify API key not configured",
        )),
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 5. Dry-run mode (exit code 0)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dry_run_edm_balance() {
    nl().args(["edm", "account", "balance", "--dry-run"])
        .env("NL_EDM_API_KEY", "test-key-abc")
        .assert()
        .success()
        .stderr(
            predicate::str::contains("dry_run")
                .and(predicate::str::contains("GET"))
                .and(predicate::str::contains("/v1/balance"))
                .and(predicate::str::contains("****...abc")),
        );
}

#[test]
fn dry_run_campaign_submit() {
    nl().args([
        "edm",
        "campaign",
        "submit",
        "--name",
        "Test",
        "--lists",
        "L1",
        "--subject",
        "Hi",
        "--from-name",
        "Sender",
        "--from-address",
        "a@b.com",
        "--html",
        "<p>hi</p>",
        "--dry-run",
    ])
    .env("NL_EDM_API_KEY", "my-secret-key")
    .assert()
    .success()
    .stderr(
        predicate::str::contains("POST")
            .and(predicate::str::contains("/v1/campaign/normal/submit")),
    );
}

#[test]
fn dry_run_sn_email_send() {
    nl().args([
        "sn",
        "email",
        "send",
        "--subject",
        "Test",
        "--from-address",
        "sender@example.com",
        "--html",
        "<p>Hello</p>",
        "--to",
        "recipient@example.com",
        "--dry-run",
    ])
    .env("NL_SN_API_KEY", "sn-api-key-xyz")
    .assert()
    .success()
    .stderr(
        predicate::str::contains("dry_run")
            .and(predicate::str::contains("POST"))
            .and(predicate::str::contains("****...xyz")),
    );
}

#[test]
fn dry_run_edm_contacts_list_groups() {
    nl().args(["edm", "contacts", "list-groups", "--dry-run"])
        .env("NL_EDM_API_KEY", "test-key-123")
        .assert()
        .success()
        .stderr(
            predicate::str::contains("GET").and(predicate::str::contains("/v1/contacts/lists")),
        );
}

#[test]
fn dry_run_edm_report_list() {
    nl().args([
        "edm",
        "report",
        "list",
        "--start-date",
        "2025-01-01",
        "--end-date",
        "2025-01-31",
        "--dry-run",
    ])
    .env("NL_EDM_API_KEY", "test-key-abc")
    .assert()
    .success()
    .stderr(predicate::str::contains("GET").and(predicate::str::contains("/v1/report/campaigns")));
}

// ══════════════════════════════════════════════════════════════════════════════
// 6. Variable syntax warning
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn edm_campaign_wrong_variable_syntax_warning() {
    // Using Surenotify-style {{name}} in an EDM campaign should trigger a warning.
    nl().args([
        "edm",
        "campaign",
        "submit",
        "--name",
        "Test",
        "--lists",
        "L1",
        "--subject",
        "Hi {{name}}",
        "--from-name",
        "Sender",
        "--from-address",
        "a@b.com",
        "--html",
        "<p>Hello</p>",
        "--dry-run",
    ])
    .env("NL_EDM_API_KEY", "test")
    .assert()
    .success()
    .stderr(
        predicate::str::contains("Warning").and(predicate::str::contains(
            "EDM API uses ${FIELD} variable syntax",
        )),
    );
}

#[test]
fn sn_email_wrong_variable_syntax_warning() {
    // Using EDM-style ${NAME} in a Surenotify email should trigger a warning.
    nl().args([
        "sn",
        "email",
        "send",
        "--subject",
        "Hello ${NAME}",
        "--from-address",
        "a@b.com",
        "--html",
        "<p>Hi</p>",
        "--to",
        "test@example.com",
        "--dry-run",
    ])
    .env("NL_SN_API_KEY", "test-key")
    .assert()
    .success()
    .stderr(
        predicate::str::contains("Warning").and(predicate::str::contains(
            "Surenotify API uses {{variable}} variable syntax",
        )),
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 7. Output format flags
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn output_format_json() {
    nl().args(["edm", "account", "balance", "--dry-run", "--format", "json"])
        .env("NL_EDM_API_KEY", "test-key")
        .assert()
        .success();
}

#[test]
fn output_format_table() {
    nl().args([
        "edm",
        "account",
        "balance",
        "--dry-run",
        "--format",
        "table",
    ])
    .env("NL_EDM_API_KEY", "test-key")
    .assert()
    .success();
}

#[test]
fn output_format_yaml() {
    nl().args(["edm", "account", "balance", "--dry-run", "--format", "yaml"])
        .env("NL_EDM_API_KEY", "test-key")
        .assert()
        .success();
}

#[test]
fn output_format_csv() {
    nl().args(["edm", "account", "balance", "--dry-run", "--format", "csv"])
        .env("NL_EDM_API_KEY", "test-key")
        .assert()
        .success();
}

#[test]
fn output_format_invalid() {
    nl().args(["edm", "account", "balance", "--dry-run", "--format", "xml"])
        .env("NL_EDM_API_KEY", "test-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'xml'"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 8. Invalid arguments
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn campaign_submit_missing_required_args() {
    // `nl edm campaign submit` with no args at all should fail (missing --name etc.)
    nl().args(["edm", "campaign", "submit"])
        .env("NL_EDM_API_KEY", "test")
        .assert()
        .failure();
}

#[test]
fn campaign_submit_partially_missing_args() {
    // Missing --subject, --from-name, --from-address, and html content
    nl().args([
        "edm", "campaign", "submit", "--name", "Test", "--lists", "L1",
    ])
    .env("NL_EDM_API_KEY", "test")
    .assert()
    .failure();
}

#[test]
fn sn_email_send_missing_required_args() {
    // `nl sn email send` with no args should fail (missing --subject etc.)
    nl().args(["sn", "email", "send"])
        .env("NL_SN_API_KEY", "test")
        .assert()
        .failure();
}

#[test]
fn sn_sms_send_missing_content() {
    // `nl sn sms send` missing the required --content flag
    nl().args(["sn", "sms", "send"])
        .env("NL_SN_API_KEY", "test")
        .assert()
        .failure();
}

// ══════════════════════════════════════════════════════════════════════════════
// 9. Config commands (no API key needed)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn config_list_succeeds() {
    nl().args(["config", "list"]).assert().success();
}

#[test]
fn config_profile_list_succeeds() {
    nl().args(["config", "profile", "list"]).assert().success();
}

// ══════════════════════════════════════════════════════════════════════════════
// 10. Conflicting arguments
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn sn_email_send_html_and_html_file_conflict() {
    nl().args([
        "sn",
        "email",
        "send",
        "--subject",
        "x",
        "--from-address",
        "a@b.com",
        "--html",
        "<p>x</p>",
        "--html-file",
        "test.html",
        "--to",
        "a@b.com",
    ])
    .env("NL_SN_API_KEY", "test")
    .assert()
    .failure()
    .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn edm_campaign_html_and_html_file_conflict() {
    nl().args([
        "edm",
        "campaign",
        "submit",
        "--name",
        "Test",
        "--lists",
        "L1",
        "--subject",
        "Hello",
        "--from-name",
        "Sender",
        "--from-address",
        "a@b.com",
        "--html",
        "<p>hi</p>",
        "--html-file",
        "test.html",
    ])
    .env("NL_EDM_API_KEY", "test")
    .assert()
    .failure()
    .stderr(predicate::str::contains("cannot be used with"));
}

// ══════════════════════════════════════════════════════════════════════════════
// 11. Quiet mode
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn quiet_mode_suppresses_stdout() {
    // In dry-run + quiet mode, there should be no stdout output.
    // The dry-run info goes to stderr only.
    nl().args(["edm", "account", "balance", "--dry-run", "-q"])
        .env("NL_EDM_API_KEY", "test-key-abc")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn quiet_mode_still_has_stderr_dry_run() {
    // Even in quiet mode, the dry-run envelope is written to stderr.
    nl().args(["edm", "account", "balance", "--dry-run", "-q"])
        .env("NL_EDM_API_KEY", "test-key-abc")
        .assert()
        .success()
        .stderr(predicate::str::contains("dry_run"));
}

// ══════════════════════════════════════════════════════════════════════════════
// Additional coverage: masked API key with different key lengths
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dry_run_masks_short_api_key() {
    // A key with 3 or fewer chars should be masked as "****" (no suffix).
    nl().args(["edm", "account", "balance", "--dry-run"])
        .env("NL_EDM_API_KEY", "ab")
        .assert()
        .success()
        .stderr(predicate::str::contains("****"));
}

#[test]
fn dry_run_masks_key_showing_suffix() {
    nl().args(["edm", "account", "balance", "--dry-run"])
        .env("NL_EDM_API_KEY", "secret-key-XYZ")
        .assert()
        .success()
        .stderr(predicate::str::contains("****...YZ"));
}

// ══════════════════════════════════════════════════════════════════════════════
// Additional coverage: Surenotify subcommands dry-run
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dry_run_sn_sms_send() {
    nl().args([
        "sn",
        "sms",
        "send",
        "--content",
        "Your code is 1234",
        "--phone",
        "912345678",
        "--country-code",
        "886",
        "--dry-run",
    ])
    .env("NL_SN_API_KEY", "sn-key-test")
    .assert()
    .success()
    .stderr(predicate::str::contains("dry_run").and(predicate::str::contains("POST")));
}

#[test]
fn dry_run_sn_domain_create() {
    nl().args([
        "sn",
        "domain",
        "create",
        "--domain",
        "mail.example.com",
        "--dry-run",
    ])
    .env("NL_SN_API_KEY", "sn-key-test")
    .assert()
    .success()
    .stderr(predicate::str::contains("dry_run"));
}

#[test]
fn dry_run_sn_webhook_list() {
    nl().args(["sn", "webhook", "list", "--dry-run"])
        .env("NL_SN_API_KEY", "sn-key-test")
        .assert()
        .success()
        .stderr(predicate::str::contains("dry_run").and(predicate::str::contains("GET")));
}

// ══════════════════════════════════════════════════════════════════════════════
// Additional coverage: unknown subcommand
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn unknown_subcommand_fails() {
    nl().args(["nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

// ══════════════════════════════════════════════════════════════════════════════
// Additional coverage: EDM dry-run produces JSON structure on stderr
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dry_run_stderr_is_valid_json_structure() {
    // The dry-run output should contain structured JSON with "dry_run" key
    // including "method", "url", and "headers".
    nl().args(["edm", "account", "balance", "--dry-run"])
        .env("NL_EDM_API_KEY", "test-key-abc")
        .assert()
        .success()
        .stderr(
            predicate::str::contains("\"dry_run\"")
                .and(predicate::str::contains("\"method\""))
                .and(predicate::str::contains("\"url\""))
                .and(predicate::str::contains("\"headers\"")),
        );
}

// ══════════════════════════════════════════════════════════════════════════════
// Additional coverage: error output is structured JSON on stderr
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn auth_error_stderr_is_json_structure() {
    // Auth errors should produce structured JSON on stderr with "error" key.
    nl().args(["edm", "account", "balance"])
        .assert()
        .code(3)
        .stderr(
            predicate::str::contains("\"error\"")
                .and(predicate::str::contains("\"type\""))
                .and(predicate::str::contains("\"auth\""))
                .and(predicate::str::contains("\"exit_code\""))
                .and(predicate::str::contains("3")),
        );
}

// ══════════════════════════════════════════════════════════════════════════════
// 12. Help text contains examples (for AI agent discoverability)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn help_top_level_has_examples() {
    nl().arg("--help").assert().success().stdout(
        predicate::str::contains("EXAMPLES:")
            .and(predicate::str::contains("ENVIRONMENT VARIABLES:")),
    );
}

#[test]
fn help_mcp_tools_has_examples() {
    nl().args(["mcp", "tools", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLE:"));
}

#[test]
fn help_mcp_call_has_examples() {
    nl().args(["mcp", "call", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn help_campaign_submit_has_examples() {
    nl().args(["edm", "campaign", "submit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"));
}

#[test]
fn help_sn_email_send_has_examples() {
    nl().args(["sn", "email", "send", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EXAMPLES:"));
}
