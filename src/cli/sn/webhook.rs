#[derive(clap::Args, Debug)]
pub struct WebhookArgs {
    #[command(subcommand)]
    pub command: WebhookCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum WebhookCommand {
    /// Create an email event webhook
    #[command(after_long_help = "\
EXAMPLE:\n  \
  nlm sn webhook create --event-type delivery --url https://example.com/hooks/email")]
    Create {
        /// Event type: delivery, open, click, bounce, complaint
        #[arg(long)]
        event_type: String,

        /// Webhook callback URL
        #[arg(long)]
        url: String,
    },

    /// List all email event webhooks
    #[command(after_long_help = "EXAMPLE:\n  nlm sn webhook list")]
    List,

    /// Delete an email event webhook by event type
    #[command(after_long_help = "EXAMPLE:\n  nlm sn webhook delete --event-type bounce")]
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
    #[command(after_long_help = "\
EXAMPLE:\n  \
  nlm sn sms-webhook create --event-type delivery --url https://example.com/hooks/sms")]
    Create {
        /// Event type: delivery, bounce
        #[arg(long)]
        event_type: String,

        /// Webhook callback URL
        #[arg(long)]
        url: String,
    },

    /// List all SMS event webhooks
    #[command(after_long_help = "EXAMPLE:\n  nlm sn sms-webhook list")]
    List,

    /// Delete an SMS event webhook by event type
    #[command(after_long_help = "EXAMPLE:\n  nlm sn sms-webhook delete --event-type bounce")]
    Delete {
        /// Event type to remove: delivery, bounce
        #[arg(long)]
        event_type: String,
    },
}
