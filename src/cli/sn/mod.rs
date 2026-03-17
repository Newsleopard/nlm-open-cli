pub mod domain;
pub mod email;
pub mod sms;
pub mod webhook;

#[derive(clap::Args, Debug)]
pub struct SnArgs {
    #[command(subcommand)]
    pub command: SnCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum SnCommand {
    /// Transactional email
    #[command(
        long_about = "Transactional email — send emails with variable substitution and query delivery events (delivered, bounced, opened)."
    )]
    Email(email::EmailArgs),

    /// SMS messaging
    #[command(
        long_about = "SMS messaging — send SMS messages to single or bulk recipients, query delivery events, and list dedicated numbers."
    )]
    Sms(sms::SmsArgs),

    /// Email webhook management
    #[command(
        long_about = "Email webhook management — create, list, and delete webhooks for email delivery event notifications."
    )]
    Webhook(webhook::WebhookArgs),

    /// SMS webhook management
    #[command(
        long_about = "SMS webhook management — create, list, and delete webhooks for SMS delivery event notifications."
    )]
    SmsWebhook(webhook::SmsWebhookArgs),

    /// Sender domain verification
    #[command(
        long_about = "Sender domain verification — create a domain, verify DNS records, and remove domains."
    )]
    Domain(domain::DomainArgs),
}
