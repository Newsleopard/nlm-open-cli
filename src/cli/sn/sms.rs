use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct SmsArgs {
    #[command(subcommand)]
    pub command: SmsCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum SmsCommand {
    /// Send an SMS message
    Send {
        /// Message content
        #[arg(long)]
        content: String,

        /// Single recipient phone number (mutually exclusive with --recipients / --recipients-file)
        #[arg(long, conflicts_with_all = ["recipients", "recipients_file"])]
        phone: Option<String>,

        /// Country code for single phone (required when --phone is used, e.g. 886)
        #[arg(long, requires = "phone")]
        country_code: Option<String>,

        /// JSON array of recipient objects
        #[arg(long, conflicts_with_all = ["phone", "recipients_file"])]
        recipients: Option<String>,

        /// File containing a JSON array of recipient objects
        #[arg(long, conflicts_with_all = ["phone", "recipients"])]
        recipients_file: Option<PathBuf>,

        /// Sender ID / from number
        #[arg(long)]
        from: Option<String>,

        /// Minutes the message remains alive for delivery
        #[arg(long)]
        alive_mins: Option<u16>,
    },

    /// Query SMS delivery events
    Events {
        /// Filter by message ID
        #[arg(long)]
        id: Option<String>,

        /// Filter by recipient phone number
        #[arg(long)]
        recipient: Option<String>,

        /// Filter by country code
        #[arg(long)]
        country_code: Option<String>,

        /// Filter events from this date (e.g. 2025-01-01)
        #[arg(long)]
        from: Option<String>,

        /// Filter events up to this date (e.g. 2025-01-31)
        #[arg(long = "to")]
        to_date: Option<String>,

        /// Filter by event status
        #[arg(long)]
        status: Option<String>,

        /// Page number
        #[arg(long)]
        page: Option<u32>,

        /// Page size
        #[arg(long)]
        size: Option<u32>,
    },

    /// List exclusive (dedicated) SMS numbers
    ExclusiveNumber,
}
