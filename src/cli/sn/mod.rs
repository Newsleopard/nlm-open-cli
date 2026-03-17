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
    Email(email::EmailArgs),

    /// SMS messaging
    Sms(sms::SmsArgs),

    /// Email webhook management
    Webhook(webhook::WebhookArgs),

    /// SMS webhook management
    SmsWebhook(webhook::SmsWebhookArgs),

    /// Sender domain verification
    Domain(domain::DomainArgs),
}
