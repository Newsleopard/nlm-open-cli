use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub command: TemplateCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum TemplateCommand {
    /// List all templates
    #[command(after_long_help = "EXAMPLE:\n  nlm edm template list")]
    List,

    /// Get a template by ID and optionally save to a file
    #[command(after_long_help = "EXAMPLES:\n  \
  nlm edm template get --id TPL001\n  \
  nlm edm template get --id TPL001 --output template.html")]
    Get {
        /// Template ID
        #[arg(long)]
        id: String,

        /// Save template HTML to this file path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    // ── MCP-backed commands ─────────────────────────────
    /// Save a campaign's content as a reusable template (via MCP)
    #[command(
        after_long_help = "EXAMPLE:\n  nlm edm template save --campaign-sn CAM12345 --name 'Monthly Newsletter Template'"
    )]
    Save {
        /// Campaign SN to save as template
        #[arg(long)]
        campaign_sn: String,

        /// Template name
        #[arg(long)]
        name: String,
    },
}
