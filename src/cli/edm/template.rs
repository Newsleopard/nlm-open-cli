use std::path::PathBuf;

#[derive(clap::Args, Debug)]
pub struct TemplateArgs {
    #[command(subcommand)]
    pub command: TemplateCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum TemplateCommand {
    /// List all templates
    List,

    /// Get a template by ID and optionally save to a file
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
    Save {
        /// Campaign SN to save as template
        #[arg(long)]
        campaign_sn: String,

        /// Template name
        #[arg(long)]
        name: String,
    },
}
