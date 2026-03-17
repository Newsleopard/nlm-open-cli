use std::path::PathBuf;

use crate::cli::CampaignSubmitFields;

#[derive(clap::Args, Debug)]
pub struct CampaignArgs {
    #[command(subcommand)]
    pub command: CampaignCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum CampaignCommand {
    /// Submit a campaign for sending
    #[command(after_long_help = "\
EXAMPLES:\n  \
  # Send immediately to list L1\n  \
  nlm edm campaign submit --name 'March Newsletter' --lists L1 \\\n    \
    --subject 'March Updates' --from-name 'ACME' --from-address news@acme.com \\\n    \
    --html-file newsletter.html\n\n  \
  # Schedule for a specific time\n  \
  nlm edm campaign submit --name 'Promo' --lists L1,L2 --exclude-lists L3 \\\n    \
    --subject 'Sale!' --from-name 'Shop' --from-address shop@acme.com \\\n    \
    --html '<h1>50% Off</h1>' --schedule scheduled \\\n    \
    --schedule-date '2025-03-20T09:00:00' --schedule-timezone 8\n\n  \
  # Dry-run to preview the request\n  \
  nlm edm campaign submit --name Test --lists L1 --subject Hi \\\n    \
    --from-name Me --from-address me@x.com --html '<p>hi</p>' --dry-run\n\n\
NOTE: EDM uses ${FIELD_NAME} variable syntax in subject/content.")]
    Submit {
        #[command(flatten)]
        fields: CampaignSubmitFields,
    },

    /// Submit a one-time campaign to contacts from a file (no stored list)
    SubmitOnce {
        /// CSV/Excel file containing recipient contacts
        #[arg(long)]
        contacts_file: PathBuf,

        /// Campaign name
        #[arg(long)]
        name: String,

        /// Email subject line
        #[arg(long)]
        subject: String,

        /// Sender display name
        #[arg(long)]
        from_name: String,

        /// Sender email address
        #[arg(long)]
        from_address: String,

        /// HTML content as inline string
        #[arg(long, conflicts_with = "html_file")]
        html: Option<String>,

        /// Path to an HTML file
        #[arg(long, conflicts_with = "html")]
        html_file: Option<PathBuf>,

        /// Footer language: chinese, english, japanese
        #[arg(long, default_value = "chinese")]
        footer_lang: String,

        /// Preheader text
        #[arg(long)]
        preheader: Option<String>,

        /// Schedule type: immediate or scheduled
        #[arg(long, default_value = "immediate")]
        schedule: String,

        /// Schedule date (e.g. 2025-01-15T09:00:00)
        #[arg(long)]
        schedule_date: Option<String>,

        /// Schedule timezone offset (e.g. 8 for UTC+8)
        #[arg(long)]
        schedule_timezone: Option<u8>,

        /// Enable Google Analytics tracking
        #[arg(long)]
        ga: bool,

        /// Enable GA e-commerce tracking
        #[arg(long)]
        ga_ecommerce: bool,

        /// Custom utm_campaign value
        #[arg(long)]
        utm_campaign: Option<String>,

        /// Custom utm_content value
        #[arg(long)]
        utm_content: Option<String>,
    },

    /// Delete one or more campaigns
    Delete {
        /// Comma-separated campaign SNs to delete
        #[arg(long)]
        sns: String,
    },

    /// Pause a sending campaign
    Pause {
        /// Campaign SN
        #[arg(long)]
        sn: String,
    },

    /// Check campaign sending status
    #[command(after_long_help = "EXAMPLE:\n  nlm edm campaign status --sn CAM12345")]
    Status {
        /// Campaign SN
        #[arg(long)]
        sn: String,
    },

    // ── MCP-backed commands ─────────────────────────────
    /// Analyze campaign performance with AI-powered suggestions (via MCP)
    #[command(
        after_long_help = "EXAMPLE:\n  nlm edm campaign analyze --sn CAM12345\n\n  \
        Returns AI-powered performance analysis with actionable suggestions."
    )]
    Analyze {
        /// Campaign SN
        #[arg(long)]
        sn: String,
    },

    /// Compare 2-5 campaigns side by side (via MCP)
    #[command(after_long_help = "EXAMPLE:\n  nlm edm campaign compare --sns CAM001 CAM002 CAM003")]
    Compare {
        /// Campaign SNs to compare (2-5)
        #[arg(long, num_args = 2..=5)]
        sns: Vec<String>,
    },

    /// Pre-flight check before sending a campaign (via MCP)
    #[command(after_long_help = "EXAMPLE:\n  nlm edm campaign preflight --sn CAM12345")]
    Preflight {
        /// Campaign SN
        #[arg(long)]
        sn: String,
    },

    /// Search campaigns by keyword or criteria (via MCP)
    #[command(after_long_help = "EXAMPLE:\n  nlm edm campaign find \"March newsletter\"")]
    Find {
        /// Search query
        query: String,
    },

    /// Get the best time to send campaigns (via MCP)
    BestTime,
}
