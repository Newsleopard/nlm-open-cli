#[derive(clap::Args, Debug)]
pub struct WebhookArgs {
    #[command(subcommand)]
    pub command: WebhookCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum WebhookCommand {
    /// Create an email event webhook
    Create {
        /// Event type: delivery, open, click, bounce, complaint
        #[arg(long)]
        event_type: String,

        /// Webhook callback URL
        #[arg(long)]
        url: String,
    },

    /// List all email event webhooks
    List,

    /// Delete an email event webhook by event type
    Delete {
        /// Event type to remove: delivery, open, click, bounce, complaint
        #[arg(long)]
        event_type: String,
    },
}

// ── SMS Webhooks ───────────────────────────────────────────────

#[derive(clap::Args, Debug)]
pub struct SmsWebhookArgs {
    #[command(subcommand)]
    pub command: SmsWebhookCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum SmsWebhookCommand {
    /// Create an SMS event webhook
    Create {
        /// Event type: delivery, bounce
        #[arg(long)]
        event_type: String,

        /// Webhook callback URL
        #[arg(long)]
        url: String,
    },

    /// List all SMS event webhooks
    List,

    /// Delete an SMS event webhook by event type
    Delete {
        /// Event type to remove: delivery, bounce
        #[arg(long)]
        event_type: String,
    },
}
