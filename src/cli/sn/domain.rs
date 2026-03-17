#[derive(clap::Args, Debug)]
pub struct DomainArgs {
    #[command(subcommand)]
    pub command: DomainCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum DomainCommand {
    /// Register a new sender domain
    #[command(after_long_help = "\
EXAMPLE:\n  \
  nlm sn domain create --domain mail.example.com")]
    Create {
        /// Domain name (e.g. mail.example.com)
        #[arg(long)]
        domain: String,
    },

    /// Verify DNS records for a sender domain
    #[command(after_long_help = "\
EXAMPLE:\n  \
  nlm sn domain verify --domain mail.example.com")]
    Verify {
        /// Domain name to verify
        #[arg(long)]
        domain: String,
    },

    /// Remove a sender domain
    #[command(after_long_help = "\
EXAMPLE:\n  \
  nlm sn domain remove --domain mail.example.com")]
    Remove {
        /// Domain name to remove
        #[arg(long)]
        domain: String,
    },
}
