//! `nlm mcp` subcommand group for agent-friendly MCP tool discovery and invocation.

#[derive(clap::Args, Debug)]
#[command(
    long_about = "MCP (Model Context Protocol) tool discovery and invocation.\n\n\
    AI agents can use 'nlm mcp tools' to list all available tools with their descriptions\n\
    and parameter schemas, then 'nlm mcp call <tool_name>' to invoke any tool.\n\n\
    Requires NL_MCP_URL to be set (default: https://mcp.newsleopard.com)."
)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum McpCommand {
    /// List all available MCP tools with descriptions
    #[command(after_long_help = "EXAMPLE:\n  nlm mcp tools\n\n  \
        Returns a JSON array of available tools, each with name, description, and inputSchema.")]
    Tools,

    /// Call any MCP tool by name
    #[command(after_long_help = "\
EXAMPLES:\n  \
  # Call with JSON string\n  \
  nlm mcp call analyze_campaign '{\"sn\":\"CAM12345\"}'\n\n  \
  # Call with key=value pairs\n  \
  nlm mcp call analyze_campaign --arg sn=CAM12345\n\n  \
  # Call with no arguments\n  \
  nlm mcp call get_best_send_time")]
    Call {
        /// Tool name (e.g. analyze_campaign, compare_campaigns)
        tool_name: String,

        /// Tool arguments as a JSON string (e.g. '{"sn":"12345"}')
        #[arg(default_value = "{}")]
        json_args: String,

        /// Alternative: pass arguments as key=value pairs (repeatable)
        #[arg(long = "arg", num_args = 1)]
        kv_args: Vec<String>,
    },
}
