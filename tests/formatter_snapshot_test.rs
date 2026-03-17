//! Snapshot tests for every formatter variant.
//!
//! Covers four formatters × multiple data shapes to detect unintended rendering
//! regressions.  Snapshots live in `tests/snapshots/`.
//!
//! Run once to generate snapshots:
//!   cargo test --test formatter_snapshot_test
//! Accept pending snapshots:
//!   cargo insta review --accept
//! or set INSTA_UPDATE=always to accept automatically:
//!   INSTA_UPDATE=always cargo test --test formatter_snapshot_test

use nl_cli::formatter::{csv_fmt, json, table, yaml};
use serde_json::json;

// ─────────────────────────────────────────────────────────────────────────────
// Shared test fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A representative EDM campaign object (single nested value).
fn campaign_object() -> serde_json::Value {
    json!({
        "id": "camp_abc123",
        "name": "Spring Sale Newsletter",
        "status": "draft",
        "subject": "Don't miss our Spring Sale!",
        "open_rate": 0.28,
        "click_rate": 0.07,
        "sent_count": 15000,
        "schedule": {
            "type": 0,
            "send_at": "2024-04-01T09:00:00Z"
        }
    })
}

/// A short contact list — an array of objects with consistent keys.
fn contact_list() -> serde_json::Value {
    json!([
        {
            "email": "alice@example.com",
            "name": "Alice Chen",
            "subscribed": true,
            "tags": ["vip", "newsletter"]
        },
        {
            "email": "bob@example.com",
            "name": "Bob Wang",
            "subscribed": false,
            "tags": ["newsletter"]
        },
        {
            "email": "carol@example.com",
            "name": "Carol Lin",
            "subscribed": true,
            "tags": []
        }
    ])
}

/// A deeply-nested configuration object to exercise dot-notation flattening.
fn nested_config() -> serde_json::Value {
    json!({
        "sender": {
            "name": "NewsLeopard",
            "email": "no-reply@newsleopard.com"
        },
        "tracking": {
            "opens": true,
            "clicks": true,
            "unsubscribes": false
        },
        "schedule": {
            "type": 1,
            "timezone": "Asia/Taipei"
        }
    })
}

/// An array of heterogeneous campaign-summary objects (different key sets).
fn campaign_summary_list() -> serde_json::Value {
    json!([
        {
            "id": "camp_001",
            "name": "Welcome Series",
            "status": "sent",
            "open_rate": 0.42
        },
        {
            "id": "camp_002",
            "name": "Promo Blast",
            "status": "draft",
            "click_rate": 0.11
        },
        {
            "id": "camp_003",
            "name": "Re-engagement",
            "status": "scheduled",
            "open_rate": 0.19,
            "click_rate": 0.05
        }
    ])
}

// ─────────────────────────────────────────────────────────────────────────────
// JSON formatter snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn json_pretty_campaign_object() {
    let output = json::format_json(&campaign_object(), false).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn json_compact_piped_contact_list() {
    let output = json::format_json(&contact_list(), true).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn json_pretty_nested_config() {
    let output = json::format_json(&nested_config(), false).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn json_ndjson_campaign_summary_list() {
    let output = json::format_ndjson(&campaign_summary_list()).unwrap();
    // Each line must be independently valid JSON.
    for line in output.lines() {
        let _: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|_| panic!("NDJSON line is not valid JSON: {line}"));
    }
    insta::assert_snapshot!(output);
}

#[test]
fn json_ndjson_single_object() {
    let output = json::format_ndjson(&campaign_object()).unwrap();
    insta::assert_snapshot!(output);
}

// ─────────────────────────────────────────────────────────────────────────────
// Table formatter snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn table_single_campaign_object() {
    let output = table::format_table(&campaign_object()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn table_contact_list_array() {
    let output = table::format_table(&contact_list()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn table_nested_config_flattened() {
    let output = table::format_table(&nested_config()).unwrap();
    // Dot-notation keys must appear in the rendered table.
    assert!(
        output.contains("sender.name"),
        "Expected flattened key 'sender.name' in table output"
    );
    insta::assert_snapshot!(output);
}

#[test]
fn table_heterogeneous_campaign_summary_list() {
    let output = table::format_table(&campaign_summary_list()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn table_empty_array() {
    let output = table::format_table(&json!([])).unwrap();
    assert_eq!(output, "(empty)");
    insta::assert_snapshot!(output);
}

// ─────────────────────────────────────────────────────────────────────────────
// YAML formatter snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn yaml_campaign_object() {
    let output = yaml::format_yaml(&campaign_object()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn yaml_contact_list_array() {
    let output = yaml::format_yaml(&contact_list()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn yaml_nested_config() {
    let output = yaml::format_yaml(&nested_config()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn yaml_page_first_no_extra_separator() {
    let output = yaml::format_yaml_page(&campaign_object(), true).unwrap();
    // Must not start with a double document-start marker.
    assert!(
        !output.starts_with("---\n---"),
        "First YAML page must not have a double --- separator"
    );
    insta::assert_snapshot!(output);
}

#[test]
fn yaml_page_subsequent_has_separator() {
    let output = yaml::format_yaml_page(&campaign_object(), false).unwrap();
    assert!(
        output.starts_with("---\n"),
        "Subsequent YAML pages must start with ---"
    );
    insta::assert_snapshot!(output);
}

// ─────────────────────────────────────────────────────────────────────────────
// CSV formatter snapshots
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn csv_contact_list_array() {
    let output = csv_fmt::format_csv(&contact_list()).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    // header + 3 data rows
    assert_eq!(lines.len(), 4, "Expected 4 CSV lines (1 header + 3 rows)");
    insta::assert_snapshot!(output);
}

#[test]
fn csv_single_campaign_object() {
    let output = csv_fmt::format_csv(&campaign_object()).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    // header + 1 data row
    assert_eq!(lines.len(), 2, "Expected 2 CSV lines (1 header + 1 row)");
    insta::assert_snapshot!(output);
}

#[test]
fn csv_nested_config_flattened() {
    let output = csv_fmt::format_csv(&nested_config()).unwrap();
    assert!(
        output.contains("sender.name"),
        "Expected flattened key 'sender.name' in CSV header"
    );
    insta::assert_snapshot!(output);
}

#[test]
fn csv_heterogeneous_campaign_summary_list() {
    let output = csv_fmt::format_csv(&campaign_summary_list()).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    // header + 3 data rows
    assert_eq!(lines.len(), 4, "Expected 4 CSV lines (1 header + 3 rows)");
    insta::assert_snapshot!(output);
}

#[test]
fn csv_page_first_includes_header() {
    let data = json!([{"id": "camp_001", "status": "sent"}]);
    let output = csv_fmt::format_csv_page(&data, true).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "id,status");
    insta::assert_snapshot!(output);
}

#[test]
fn csv_page_subsequent_omits_header() {
    let data = json!([{"id": "camp_001", "status": "sent"}]);
    let output = csv_fmt::format_csv_page(&data, false).unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines.len(), 1, "Subsequent CSV page must not include header");
    insta::assert_snapshot!(output);
}
