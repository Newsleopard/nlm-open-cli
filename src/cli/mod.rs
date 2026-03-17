use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod edm;
pub mod mcp;
pub mod sn;

// ── Top-level CLI ──────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(name = "nl", about = "Newsleopard EDM & Surenotify CLI", version)]
pub struct NlCli {
    #[command(subcommand)]
    pub command: Command,

    /// Output format
    #[arg(long, global = true, default_value = "json", env = "NL_FORMAT")]
    pub format: OutputFormat,

    /// Config profile
    #[arg(long, global = true, default_value = "default", env = "NL_PROFILE")]
    pub profile: String,

    /// Preview request without executing
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Verbose output (-v summary, -vv full)
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// EDM API commands
    Edm(Box<edm::EdmArgs>),

    /// Surenotify API commands
    Sn(sn::SnArgs),

    /// MCP tool discovery and invocation (agent-friendly)
    Mcp(mcp::McpArgs),

    /// Configuration management
    Config(ConfigArgs),

    /// High-level orchestration commands
    #[command(alias = "x")]
    Helper(HelperArgs),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Yaml,
    Csv,
}

// ── Config ─────────────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Interactive first-time setup
    Init,

    /// Set a configuration value
    Set {
        /// Configuration key (e.g. edm_api_key, sn_api_key, default_format)
        key: String,

        /// Value to set
        value: String,

        /// Profile to update (defaults to current profile)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Get a configuration value
    Get {
        /// Configuration key to retrieve
        key: String,

        /// Profile to read from (defaults to current profile)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Show all settings (API keys are masked)
    List,

    /// Profile management
    Profile(ProfileArgs),
}

#[derive(clap::Args, Debug)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommand {
    /// Create a new profile
    Create {
        /// Profile name
        name: String,
    },

    /// List all profiles
    List,

    /// Delete a profile
    Delete {
        /// Profile name to delete
        name: String,
    },
}

// ── Helper (orchestration) ─────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct HelperArgs {
    #[command(subcommand)]
    pub command: HelperCommand,
}

#[derive(Subcommand, Debug)]
pub enum HelperCommand {
    /// Submit a campaign and optionally wait for completion
    CampaignSend {
        #[command(flatten)]
        campaign: Box<CampaignSubmitFields>,

        /// Wait for the campaign to finish sending
        #[arg(long)]
        wait: bool,
    },

    /// Import contacts from a file and poll until the import completes
    ImportAndWait {
        /// Contact list SN
        #[arg(long)]
        list_sn: String,

        /// CSV/Excel file to import
        #[arg(long)]
        file: PathBuf,

        /// Maximum seconds to wait for import completion
        #[arg(long)]
        timeout: Option<u64>,

        /// Seconds between status polls
        #[arg(long)]
        poll_interval: Option<u64>,
    },

    /// Export a campaign report and download the file
    ReportDownload {
        /// Campaign SN
        #[arg(long)]
        sn: String,

        /// Output file path
        #[arg(long)]
        output: PathBuf,
    },

    /// Set up a sender domain with DNS records and optional auto-verify
    DomainSetup {
        /// Domain to set up (e.g. mail.example.com)
        #[arg(long)]
        domain: String,

        /// Seconds to wait before attempting automatic verification
        #[arg(long)]
        auto_verify_after: Option<u64>,
    },
}

// ── Shared campaign submit fields (used by campaign::Submit and helper::CampaignSend) ──

#[derive(clap::Args, Debug)]
pub struct CampaignSubmitFields {
    /// Campaign name
    #[arg(long)]
    pub name: String,

    /// Comma-separated list SNs to send to
    #[arg(long)]
    pub lists: String,

    /// Email subject line
    #[arg(long)]
    pub subject: String,

    /// Sender display name
    #[arg(long)]
    pub from_name: String,

    /// Sender email address
    #[arg(long)]
    pub from_address: String,

    /// HTML content as inline string
    #[arg(long, conflicts_with = "html_file")]
    pub html: Option<String>,

    /// Path to an HTML file
    #[arg(long, conflicts_with = "html")]
    pub html_file: Option<PathBuf>,

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
