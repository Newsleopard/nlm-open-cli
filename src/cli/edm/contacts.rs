use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct ContactsArgs {
    #[command(subcommand)]
    pub command: ContactsCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum ContactsCommand {
    /// Create a new contact group
    CreateGroup {
        /// Group name
        #[arg(long)]
        name: String,
    },

    /// List all contact groups
    ListGroups {
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,

        /// Page size
        #[arg(long)]
        size: Option<u32>,

        /// Fetch all pages and output as NDJSON
        #[arg(long)]
        page_all: bool,
    },

    /// Import contacts from a CSV/Excel file
    ImportFile {
        /// Target contact list SN
        #[arg(long)]
        list_sn: String,

        /// CSV or Excel file to import
        #[arg(long)]
        file: PathBuf,

        /// Webhook URL to notify on completion
        #[arg(long)]
        webhook_url: Option<String>,

        /// Wait and poll until the import completes
        #[arg(long)]
        wait: bool,

        /// Seconds between status polls (used with --wait)
        #[arg(long)]
        poll_interval: Option<u64>,
    },

    /// Import contacts from inline CSV text or a CSV file body
    ImportText {
        /// Target contact list SN
        #[arg(long)]
        list_sn: String,

        /// Inline CSV text (mutually exclusive with --csv-file)
        #[arg(long, conflicts_with = "csv_file")]
        csv_text: Option<String>,

        /// Path to CSV file to read as text body (mutually exclusive with --csv-text)
        #[arg(long, conflicts_with = "csv_text")]
        csv_file: Option<PathBuf>,

        /// Webhook URL to notify on completion
        #[arg(long)]
        webhook_url: Option<String>,
    },

    /// Check the status of a contact import job
    ImportStatus {
        /// Import job SN
        #[arg(long)]
        import_sn: String,
    },

    /// Remove contacts matching a filter condition
    Remove {
        /// Target contact list SN
        #[arg(long)]
        list_sn: String,

        /// Field to filter on (e.g. email, name)
        #[arg(long)]
        field: String,

        /// Comparison operator (e.g. eq, contains)
        #[arg(long)]
        operator: String,

        /// Value to match
        #[arg(long)]
        value: String,
    },

    // ── MCP-backed commands ─────────────────────────────
    /// Top-performing contact lists ranked by engagement (via MCP)
    TopLists {
        /// Maximum number of lists to return
        #[arg(long)]
        limit: Option<u32>,
    },
}
