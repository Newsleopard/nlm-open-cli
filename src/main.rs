use clap::Parser;
use std::process;
use tracing_subscriber::EnvFilter;

use nlm_cli::cli::NlCli;
use nlm_cli::executor;

#[tokio::main]
async fn main() {
    // Initialize tracing with RUST_LOG env filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_target(false)
        .init();

    let cli = NlCli::parse();

    match executor::execute(cli).await {
        Ok(()) => {}
        Err(e) => {
            e.to_json_stderr();
            process::exit(e.exit_code());
        }
    }
}
