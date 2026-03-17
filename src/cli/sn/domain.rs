#[derive(clap::Args, Debug)]
pub struct DomainArgs {
    #[command(subcommand)]
    pub command: DomainCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum DomainCommand {
    /// Register a new sender domain
    Create {
        /// Domain name (e.g. mail.example.com)
        #[arg(long)]
        domain: String,
    },

    /// Verify DNS records for a sender domain
    Verify {
        /// Domain name to verify
        #[arg(long)]
        domain: String,
    },

    /// Remove a sender domain
    Remove {
        /// Domain name to remove
        #[arg(long)]
        domain: String,
    },
}
