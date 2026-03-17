use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct ReportArgs {
    #[command(subcommand)]
    pub command: ReportCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum ReportCommand {
    /// List campaign reports within a date range
    #[command(after_long_help = "EXAMPLE:\n  \
        nlm edm report list --start-date 2025-01-01 --end-date 2025-01-31")]
    List {
        /// Start date (e.g. 2025-01-01)
        #[arg(long)]
        start_date: String,

        /// End date (e.g. 2025-01-31)
        #[arg(long)]
        end_date: String,
    },

    /// Get metrics for one or more campaigns
    #[command(after_long_help = "EXAMPLE:\n  nlm edm report metrics --sns CAM001,CAM002")]
    Metrics {
        /// Comma-separated campaign SNs
        #[arg(long)]
        sns: String,
    },

    /// Export a campaign report (triggers async export)
    #[command(after_long_help = "EXAMPLES:\n  \
        nlm edm report export --sn CAM12345\n  \
        nlm edm report export --sn CAM12345 --wait --output report.csv\n\n\
NOTE: Report export is rate-limited to 1 request per 10 seconds.")]
    Export {
        /// Campaign SN
        #[arg(long)]
        sn: String,

        /// Wait for the export to complete and download
        #[arg(long)]
        wait: bool,

        /// Output file path (used with --wait)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Get the download link for an exported report
    #[command(after_long_help = "EXAMPLE:\n  nlm edm report download-link --sn CAM12345")]
    DownloadLink {
        /// Campaign SN
        #[arg(long)]
        sn: String,
    },

    // ── MCP-backed commands ─────────────────────────────
    /// Recent campaigns performance summary (via MCP)
    #[command(after_long_help = "EXAMPLE:\n  nlm edm report summary --days 7")]
    Summary {
        /// Number of days to look back (default: 30)
        #[arg(long, default_value = "30")]
        days: u32,
    },

    /// Per-link click breakdown for a campaign (via MCP)
    #[command(after_long_help = "EXAMPLE:\n  nlm edm report clicks --sn CAM12345")]
    Clicks {
        /// Campaign SN
        #[arg(long)]
        sn: String,
    },
}
