use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct AutomationArgs {
    #[command(subcommand)]
    pub command: AutomationCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum AutomationCommand {
    /// Trigger an automation workflow
    Trigger {
        /// Workflow identifier
        #[arg(long)]
        workflow: String,

        /// Event name to trigger
        #[arg(long)]
        event: String,

        /// Comma-separated recipient emails (mutually exclusive with --recipients-file)
        #[arg(long, conflicts_with = "recipients_file")]
        recipients: Option<String>,

        /// File containing recipient emails, one per line (mutually exclusive with --recipients)
        #[arg(long, conflicts_with = "recipients")]
        recipients_file: Option<PathBuf>,
    },
}
