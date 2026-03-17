use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct EmailArgs {
    #[command(subcommand)]
    pub command: EmailCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum EmailCommand {
    /// Send a transactional email
    Send {
        /// Email subject line
        #[arg(long)]
        subject: String,

        /// Sender email address
        #[arg(long)]
        from_address: String,

        /// Sender display name
        #[arg(long)]
        from_name: Option<String>,

        /// HTML content as inline string
        #[arg(long, conflicts_with = "html_file")]
        html: Option<String>,

        /// Path to an HTML file
        #[arg(long, conflicts_with = "html")]
        html_file: Option<PathBuf>,

        /// Comma-separated simple recipient email addresses
        #[arg(long, conflicts_with_all = ["recipients", "recipients_file"])]
        to: Option<String>,

        /// JSON array of recipient objects (e.g. [{"address":"a@b.com","name":"A","substitutions":{}}])
        #[arg(long, conflicts_with_all = ["to", "recipients_file"])]
        recipients: Option<String>,

        /// File containing a JSON array of recipient objects
        #[arg(long, conflicts_with_all = ["to", "recipients"])]
        recipients_file: Option<PathBuf>,

        /// Unsubscribe link URL
        #[arg(long)]
        unsubscribe_link: Option<String>,
    },

    /// Query transactional email events
    Events {
        /// Filter by message ID (mutually exclusive with --recipient)
        #[arg(long, conflicts_with = "recipient")]
        id: Option<String>,

        /// Filter by recipient email (mutually exclusive with --id)
        #[arg(long, conflicts_with = "id")]
        recipient: Option<String>,

        /// Filter events from this date (e.g. 2025-01-01)
        #[arg(long)]
        from: Option<String>,

        /// Filter events up to this date (e.g. 2025-01-31)
        #[arg(long = "to")]
        to_date: Option<String>,

        /// Filter by event status (e.g. delivered, bounced, opened)
        #[arg(long)]
        status: Option<String>,

        /// Page number
        #[arg(long)]
        page: Option<u32>,

        /// Page size
        #[arg(long)]
        size: Option<u32>,
    },
}
