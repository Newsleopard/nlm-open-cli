use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct ContactsArgs {
    #[command(subcommand)]
    pub command: ContactsCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum ContactsCommand {
    /// Create a new contact group
    #[command(
        after_long_help = "EXAMPLE:\n  nlm edm contacts create-group --name 'VIP Customers'"
    )]
    CreateGroup {
        /// Group name
        #[arg(long)]
        name: String,
    },

    /// List all contact groups
    #[command(after_long_help = "EXAMPLES:\n  \
        nlm edm contacts list-groups                    # First page, JSON\n  \
        nlm edm contacts list-groups --format table     # Pretty table\n  \
        nlm edm contacts list-groups --page-all         # Stream all pages as NDJSON")]
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
    #[command(after_long_help = "EXAMPLES:\n  \
        nlm edm contacts import-file --list-sn L1 --file contacts.csv\n  \
        nlm edm contacts import-file --list-sn L1 --file contacts.csv --wait")]
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
    #[command(after_long_help = "EXAMPLES:\n  \
  nlm edm contacts import-text --list-sn L1 --csv-text 'email,name\\na@b.com,Alice'\n  \
  nlm edm contacts import-text --list-sn L1 --csv-file contacts.csv")]
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
    #[command(after_long_help = "EXAMPLE:\n  nlm edm contacts import-status --import-sn IMP12345")]
    ImportStatus {
        /// Import job SN
        #[arg(long)]
        import_sn: String,
    },

    /// Remove contacts matching a filter condition
    #[command(after_long_help = "\
EXAMPLE:\n  \
  nlm edm contacts remove --list-sn L1 --field email \\\n    \
    --operator eq --value old@example.com")]
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
    #[command(after_long_help = "EXAMPLE:\n  nlm edm contacts top-lists --limit 5")]
    TopLists {
        /// Maximum number of lists to return
        #[arg(long)]
        limit: Option<u32>,
    },
}
