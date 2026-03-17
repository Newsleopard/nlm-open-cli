#[derive(clap::Args, Debug)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum AccountCommand {
    /// Show account balance (email and SMS credits)
    #[command(after_long_help = "EXAMPLES:\n  \
  nlm edm account balance\n  \
  nlm edm account balance --format table")]
    Balance,
}
