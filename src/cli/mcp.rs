//! `nl mcp` subcommand group for agent-friendly MCP tool discovery and invocation.

#[derive(clap::Args, Debug)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum McpCommand {
    /// List all available MCP tools with descriptions
    Tools,

    /// Call any MCP tool by name
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
