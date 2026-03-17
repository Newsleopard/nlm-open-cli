use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct AbTestArgs {
    #[command(subcommand)]
    pub command: AbTestCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum AbTestCommand {
    /// Submit an A/B test campaign
    Submit {
        #[command(flatten)]
        fields: AbTestSubmitFields,
    },

    /// Submit a one-time A/B test campaign to contacts from a file (no stored list)
    SubmitOnce {
        /// CSV/Excel file containing recipient contacts
        #[arg(long)]
        contacts_file: PathBuf,

        #[command(flatten)]
        fields: AbTestSubmitFields,
    },
}

/// Shared fields for A/B test submit and submit-once
#[derive(clap::Args, Debug)]
pub struct AbTestSubmitFields {
    /// Campaign name
    #[arg(long)]
    pub name: String,

    /// Comma-separated list SNs to send to
    #[arg(long)]
    pub lists: String,

    /// What to test: subject, sender, or content
    #[arg(long)]
    pub test_on: String,

    /// Percentage of recipients for the test phase (e.g. 20)
    #[arg(long)]
    pub proportion: u8,

    /// Duration of the test phase
    #[arg(long)]
    pub test_duration: u32,

    /// Unit for test duration: hours or days
    #[arg(long)]
    pub test_unit: String,

    // ── Subject test variant fields ──
    /// Subject line for variant A (used when --test-on subject)
    #[arg(long)]
    pub subject_a: Option<String>,

    /// Subject line for variant B (used when --test-on subject)
    #[arg(long)]
    pub subject_b: Option<String>,

    // ── Sender test variant fields ──
    /// Sender display name for variant A (used when --test-on sender)
    #[arg(long)]
    pub from_name_a: Option<String>,

    /// Sender email address for variant A (used when --test-on sender)
    #[arg(long)]
    pub from_address_a: Option<String>,

    /// Sender display name for variant B (used when --test-on sender)
    #[arg(long)]
    pub from_name_b: Option<String>,

    /// Sender email address for variant B (used when --test-on sender)
    #[arg(long)]
    pub from_address_b: Option<String>,

    // ── Content test variant fields ──
    /// HTML file for variant A (used when --test-on content)
    #[arg(long)]
    pub html_content_a_file: Option<PathBuf>,

    /// HTML file for variant B (used when --test-on content)
    #[arg(long)]
    pub html_content_b_file: Option<PathBuf>,

    // ── Common fields (used when not testing that dimension) ──
    /// Email subject (used when --test-on is sender or content)
    #[arg(long)]
    pub subject: Option<String>,

    /// Sender display name (used when --test-on is subject or content)
    #[arg(long)]
    pub from_name: Option<String>,

    /// Sender email address (used when --test-on is subject or content)
    #[arg(long)]
    pub from_address: Option<String>,

    /// HTML content as inline string (used when --test-on is subject or sender)
    #[arg(long, conflicts_with = "html_file")]
    pub html: Option<String>,

    /// Path to an HTML file (used when --test-on is subject or sender)
    #[arg(long, conflicts_with = "html")]
    pub html_file: Option<PathBuf>,

    // ── Shared configuration ──
    /// Footer language: chinese, english, japanese
    #[arg(long, default_value = "chinese")]
    pub footer_lang: String,

    /// Preheader text
    #[arg(long)]
    pub preheader: Option<String>,

    /// Comma-separated list SNs to exclude
    #[arg(long)]
    pub exclude_lists: Option<String>,

    /// Schedule type: immediate or scheduled
    #[arg(long, default_value = "immediate")]
    pub schedule: String,

    /// Schedule date (e.g. 2025-01-15T09:00:00)
    #[arg(long)]
    pub schedule_date: Option<String>,

    /// Schedule timezone offset (e.g. 8 for UTC+8)
    #[arg(long)]
    pub schedule_timezone: Option<u8>,

    /// Enable Google Analytics tracking
    #[arg(long)]
    pub ga: bool,

    /// Enable GA e-commerce tracking
    #[arg(long)]
    pub ga_ecommerce: bool,

    /// Custom utm_campaign value
    #[arg(long)]
    pub utm_campaign: Option<String>,

    /// Custom utm_content value
    #[arg(long)]
    pub utm_content: Option<String>,
}
