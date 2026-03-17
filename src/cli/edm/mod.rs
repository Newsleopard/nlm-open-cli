pub mod ab_test;
pub mod account;
pub mod automation;
pub mod campaign;
pub mod contacts;
pub mod report;
pub mod template;

#[derive(clap::Args, Debug)]
pub struct EdmArgs {
    #[command(subcommand)]
    pub command: EdmCommand,
}

#[derive(clap::Subcommand, Debug)]
pub enum EdmCommand {
    /// Contact group management
    #[command(
        long_about = "Contact group management — create groups, list groups, import contacts from files or CSV text, check import status, and remove contacts."
    )]
    Contacts(contacts::ContactsArgs),

    /// Campaign management
    #[command(
        long_about = "Email campaign management — submit, check status, pause, delete campaigns.\nAlso includes MCP-powered commands: analyze, compare, preflight, find, best-time."
    )]
    Campaign(Box<campaign::CampaignArgs>),

    /// A/B testing campaigns
    #[command(
        long_about = "A/B test campaign management — submit A/B tests with multiple subject/content variants and control the test ratio."
    )]
    AbTest(Box<ab_test::AbTestArgs>),

    /// Campaign reports
    #[command(
        long_about = "Campaign reports — list reports by date range, get metrics, export reports, and download exported files.\nIncludes MCP-powered summary and click analysis."
    )]
    Report(report::ReportArgs),

    /// Template management
    #[command(
        long_about = "Template management — list available templates and get template details by SN."
    )]
    Template(template::TemplateArgs),

    /// Automation scripts
    #[command(
        long_about = "Automation scripts — trigger automation workflows by script SN with optional custom fields."
    )]
    Automation(automation::AutomationArgs),

    /// Account information
    #[command(long_about = "Account information — check account balance and quota.")]
    Account(account::AccountArgs),
}
