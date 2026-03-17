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
    Contacts(contacts::ContactsArgs),

    /// Campaign management
    Campaign(Box<campaign::CampaignArgs>),

    /// A/B testing campaigns
    AbTest(Box<ab_test::AbTestArgs>),

    /// Campaign reports
    Report(report::ReportArgs),

    /// Template management
    Template(template::TemplateArgs),

    /// Automation scripts
    Automation(automation::AutomationArgs),

    /// Account information
    Account(account::AccountArgs),
}
