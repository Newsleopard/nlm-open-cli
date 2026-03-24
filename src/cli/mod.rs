use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

pub mod edm;
pub mod mcp;
pub mod sn;

// ── Top-level CLI ──────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "nlm",
    about = "Newsleopard EDM & Surenotify CLI",
    long_about = "CLI for the Newsleopard email marketing (EDM) API and Surenotify transactional messaging API.\n\n\
        Covers 34 API endpoints: campaign management, contacts, reports, templates, A/B tests,\n\
        transactional email/SMS, webhooks, domain verification, and MCP tool discovery.\n\n\
        All output is machine-parseable (JSON/Table/YAML/CSV). Errors are JSON on stderr with typed exit codes (0-5).",
    after_long_help = "\
EXAMPLES:\n  \
  nlm config init                          # First-time API key setup\n  \
  nlm edm contacts list-groups             # List contact groups\n  \
  nlm edm campaign submit --dry-run ...    # Preview a campaign submit\n  \
  nlm sn email send --to a@b.com ...       # Send transactional email\n  \
  nlm mcp tools                            # Discover MCP tools (for AI agents)\n\n\
ENVIRONMENT VARIABLES:\n  \
  NL_EDM_API_KEY    EDM API key (overrides config file)\n  \
  NL_SN_API_KEY     Surenotify API key (overrides config file)\n  \
  NL_FORMAT         Default output format (json|table|yaml|csv)\n  \
  NL_PROFILE        Config profile name (default: \"default\")\n  \
  NL_MCP_URL        MCP server base URL (nlm uses the /mcp endpoint)",
    version
)]
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
    #[command(
        long_about = "Newsleopard EDM API — manage campaigns, contacts, reports, templates, A/B tests, automations, and account info (20 endpoints)."
    )]
    Edm(Box<edm::EdmArgs>),

    /// Surenotify API commands
    #[command(
        long_about = "Surenotify transactional messaging API — send emails and SMS, manage webhooks, and verify sender domains (14 endpoints)."
    )]
    Sn(sn::SnArgs),

    /// MCP tool discovery and invocation (agent-friendly)
    #[command(
        long_about = "MCP (Model Context Protocol) tool discovery and invocation.\n\n\
        AI agents can use 'nlm mcp tools' to list all available tools with their descriptions\n\
        and parameter schemas, then 'nlm mcp call <tool_name>' to invoke any tool.\n\n\
        Requires NL_MCP_URL to be configured (default base URL: https://mcp.newsleopard.com; nlm sends requests to /mcp)."
    )]
    Mcp(mcp::McpArgs),

    /// Configuration management
    #[command(
        long_about = "Configuration management — set up API keys, manage profiles, and view settings.\n\n\
        Config file: ~/.config/nl/config.toml (permissions: 600)."
    )]
    Config(ConfigArgs),

    /// High-level orchestration commands
    #[command(
        alias = "x",
        long_about = "High-level orchestration workflows that combine multiple API calls into single operations.\n\n\
        Examples: submit campaign + wait for completion, import contacts + poll status, export report + download file."
    )]
    Helper(HelperArgs),

    /// Generate AI agent skill files for the nlm CLI
    #[command(
        name = "generate-skills",
        long_about = "Generate AI agent skill files (SKILL.md) that teach Claude Code and other AI agents\n\
        how to use nlm's 34 API endpoints and 4 helper workflows.\n\n\
        Writes skills/{name}/SKILL.md files and an optional docs/skills.md index.\n\
        Content follows the openclaw frontmatter format for cross-tool compatibility."
    )]
    GenerateSkills {
        /// Output directory for skill files
        #[arg(long, default_value = "skills")]
        output_dir: PathBuf,

        /// Also generate docs/skills.md index
        #[arg(long, default_value_t = true)]
        index: bool,
    },
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
    #[command(after_long_help = "EXAMPLE:\n  nlm config init\n\n  \
        Guides you through setting EDM and Surenotify API keys for the default profile.")]
    Init,

    /// Set a configuration value
    #[command(after_long_help = "EXAMPLES:\n  \
        nlm config set edm_api_key \"your-key\"\n  \
        nlm config set default_format table\n  \
        nlm config set sn_api_key \"key\" --profile staging")]
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
    #[command(after_long_help = "EXAMPLE:\n  \
        nlm helper campaign-send --name 'Newsletter' --lists L1 \\\n    \
        --subject 'News' --from-name ACME --from-address news@acme.com \\\n    \
        --html-file content.html --wait")]
    CampaignSend {
        #[command(flatten)]
        campaign: Box<CampaignSubmitFields>,

        /// Wait for the campaign to finish sending
        #[arg(long)]
        wait: bool,
    },

    /// Import contacts from a file and poll until the import completes
    #[command(after_long_help = "EXAMPLE:\n  \
        nlm helper import-and-wait --list-sn L1 --file contacts.csv --timeout 300")]
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
    #[command(after_long_help = "EXAMPLE:\n  \
        nlm helper report-download --sn CAM12345 --output report.csv")]
    ReportDownload {
        /// Campaign SN
        #[arg(long)]
        sn: String,

        /// Output file path
        #[arg(long)]
        output: PathBuf,
    },

    /// Set up a sender domain with DNS records and optional auto-verify
    #[command(after_long_help = "EXAMPLE:\n  \
        nlm helper domain-setup --domain mail.example.com --auto-verify-after 60\n\n  \
        Creates the domain, displays required DNS records, then optionally waits and verifies.")]
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
