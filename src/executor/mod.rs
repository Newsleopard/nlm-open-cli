//! Executor layer: routes parsed CLI args to client calls and formatted output.
//!
//! This module is the central dispatch point. It:
//! 1. Handles config commands directly (no API client needed)
//! 2. Loads configuration and creates the appropriate API client
//! 3. Dispatches to the correct EDM, Surenotify, or helper sub-executor
//! 4. Formats and prints the result

use std::io::IsTerminal;
use std::path::Path;

use regex::Regex;
use serde_json::Value;

use crate::cli::edm::ab_test::{AbTestCommand, AbTestSubmitFields};
use crate::cli::edm::account::AccountCommand;
use crate::cli::edm::automation::AutomationCommand;
use crate::cli::edm::campaign::CampaignCommand;
use crate::cli::edm::contacts::ContactsCommand;
use crate::cli::edm::report::ReportCommand;
use crate::cli::edm::template::TemplateCommand;
use crate::cli::edm::EdmCommand;
use crate::cli::mcp::McpCommand;
use crate::cli::sn::domain::DomainCommand;
use crate::cli::sn::email::EmailCommand;
use crate::cli::sn::sms::SmsCommand;
use crate::cli::sn::webhook::{SmsWebhookCommand, WebhookCommand};
use crate::cli::sn::SnCommand;
use crate::cli::{
    CampaignSubmitFields, Command, ConfigCommand, HelperCommand, NlCli, OutputFormat,
    ProfileCommand,
};
use crate::client::edm::EdmClient;
use crate::client::mcp::McpClient;
use crate::client::surenotify::SurenotifyClient;
use crate::client::ApiClient;
use crate::config::{self, ResolvedConfig};
use crate::error::NlError;
use crate::formatter::{self, Format};
use crate::helpers;
use crate::types::edm::*;
use crate::types::surenotify::*;

/// Main dispatch: takes the fully parsed CLI and executes the appropriate command.
pub async fn execute(cli: NlCli) -> Result<(), NlError> {
    // Handle config commands directly — they don't need an API client.
    if let Command::Config(ref cfg) = cli.command {
        return execute_config(&cfg.command);
    }

    // Handle generate-skills — no API client or config needed.
    if let Command::GenerateSkills {
        ref output_dir,
        index,
    } = cli.command
    {
        return crate::skills_generator::generate(output_dir, index);
    }

    let config = config::load(&cli.profile)?;
    let client = ApiClient::new(cli.dry_run, cli.verbose);
    let format = convert_format(cli.format);
    let is_piped = !std::io::stdout().is_terminal();

    let result: Value = match cli.command {
        Command::Edm(edm) => {
            let edm_client = EdmClient::new(&client, config.edm_api_key()?);
            execute_edm(edm.command, &edm_client, &config, &client).await?
        }
        Command::Sn(sn) => {
            let sn_client = SurenotifyClient::new(&client, config.sn_api_key()?);
            execute_sn(sn.command, &sn_client).await?
        }
        Command::Mcp(mcp) => execute_mcp(mcp.command, &config, &client).await?,
        Command::Helper(helper) => execute_helper(helper.command, &config, &client).await?,
        Command::Config(_) | Command::GenerateSkills { .. } => unreachable!(),
    };

    if !cli.quiet {
        let output = formatter::format_output(&result, format, is_piped)?;
        println!("{output}");
    }
    Ok(())
}

/// Converts the CLI `OutputFormat` to the formatter's `Format`.
fn convert_format(f: OutputFormat) -> Format {
    match f {
        OutputFormat::Json => Format::Json,
        OutputFormat::Table => Format::Table,
        OutputFormat::Yaml => Format::Yaml,
        OutputFormat::Csv => Format::Csv,
    }
}

// ── Config commands (no API client needed) ──────────────────────────────────

fn execute_config(cmd: &ConfigCommand) -> Result<(), NlError> {
    match cmd {
        ConfigCommand::Init => config::init_interactive(),
        ConfigCommand::Set {
            key,
            value,
            profile,
        } => config::set_value(key, value, profile.as_deref()),
        ConfigCommand::Get { key, profile } => {
            let val = config::get_value(key, profile.as_deref())?;
            println!("{val}");
            Ok(())
        }
        ConfigCommand::List => {
            let output = config::list_all()?;
            println!("{output}");
            Ok(())
        }
        ConfigCommand::Profile(p) => match &p.command {
            ProfileCommand::Create { name } => config::create_profile(name),
            ProfileCommand::List => {
                let profiles = config::profile_list()?;
                for p in profiles {
                    println!("{p}");
                }
                Ok(())
            }
            ProfileCommand::Delete { name } => config::delete_profile(name),
        },
    }
}

// ── MCP commands ────────────────────────────────────────────────────────────

async fn execute_mcp(
    cmd: McpCommand,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());

    match cmd {
        McpCommand::Tools => {
            // Show a spinner on stderr so humans/agents see activity.
            let spinner = if std::io::stderr().is_terminal() {
                let pb = indicatif::ProgressBar::new_spinner();
                pb.set_message("Discovering MCP tools...");
                pb.enable_steady_tick(std::time::Duration::from_millis(120));
                Some(pb)
            } else {
                None
            };

            let result = mcp.list_tools().await;

            if let Some(pb) = spinner {
                match &result {
                    Ok(tools) => pb.finish_with_message(format!("Found {} tools", tools.len())),
                    Err(_) => pb.finish_with_message("Failed"),
                }
            }

            let tools = result?;
            Ok(serde_json::to_value(tools)?)
        }

        McpCommand::Call {
            tool_name,
            json_args,
            kv_args,
        } => {
            let args = if !kv_args.is_empty() {
                crate::client::mcp::parse_kv_args(&kv_args)?
            } else {
                serde_json::from_str(&json_args)
                    .map_err(|e| NlError::Validation(format!("Invalid JSON arguments: {e}")))?
            };
            mcp.call_tool(&tool_name, args).await
        }
    }
}

// ── EDM commands ────────────────────────────────────────────────────────────

async fn execute_edm(
    cmd: EdmCommand,
    edm: &EdmClient<'_>,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    match cmd {
        EdmCommand::Contacts(args) => execute_contacts(args.command, edm, config, client).await,
        EdmCommand::Campaign(args) => execute_campaign(args.command, edm, config, client).await,
        EdmCommand::AbTest(args) => execute_ab_test(args.command, edm).await,
        EdmCommand::Report(args) => execute_report(args.command, edm, config, client).await,
        EdmCommand::Template(args) => execute_template(args.command, edm, config, client).await,
        EdmCommand::Automation(args) => execute_automation(args.command, edm).await,
        EdmCommand::Account(args) => execute_account(args.command, edm).await,
    }
}

async fn execute_contacts(
    cmd: ContactsCommand,
    edm: &EdmClient<'_>,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    match cmd {
        ContactsCommand::CreateGroup { name } => edm.create_group(&name).await,

        ContactsCommand::ListGroups {
            page,
            size,
            page_all,
        } => {
            if page_all {
                // Fetch all pages and return as a JSON array.
                let mut all_groups = Vec::new();
                let mut current_page = 1u32;
                let page_size = size.unwrap_or(100);

                loop {
                    let resp = edm.list_groups(Some(current_page), Some(page_size)).await?;
                    if let Some(groups) = resp["groups"].as_array() {
                        if groups.is_empty() {
                            break;
                        }
                        all_groups.extend(groups.clone());
                        // Check if we've reached the last page.
                        let total = resp["pageInfo"]["total"].as_u64().unwrap_or(0);
                        if (current_page as u64) * (page_size as u64) >= total {
                            break;
                        }
                        current_page += 1;
                    } else {
                        break;
                    }
                }
                Ok(Value::Array(all_groups))
            } else {
                edm.list_groups(page, size).await
            }
        }

        ContactsCommand::ImportFile {
            list_sn,
            file,
            webhook_url,
            wait,
            poll_interval,
        } => {
            if wait {
                helpers::import_wait::execute(
                    &list_sn,
                    &file,
                    None, // no explicit timeout on this command
                    poll_interval,
                    edm,
                )
                .await
            } else {
                edm.import_file(&list_sn, &file, webhook_url.as_deref())
                    .await
            }
        }

        ContactsCommand::ImportText {
            list_sn,
            csv_text,
            csv_file,
            webhook_url,
        } => {
            let text = resolve_csv_text(csv_text, csv_file.as_deref())?;
            let request = ImportTextRequest {
                csv_text: text,
                webhook_url,
            };
            edm.import_text(&list_sn, &request).await
        }

        ContactsCommand::ImportStatus { import_sn } => edm.import_status(&import_sn).await,

        ContactsCommand::Remove {
            list_sn,
            field,
            operator,
            value,
        } => {
            let request = RemoveContactsRequest {
                field,
                operator,
                value,
            };
            edm.remove_contacts(&list_sn, &request).await
        }

        ContactsCommand::TopLists { limit } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            let mut args = serde_json::Map::new();
            if let Some(n) = limit {
                args.insert("limit".to_string(), serde_json::json!(n));
            }
            mcp.call_tool("get_top_lists", Value::Object(args)).await
        }
    }
}

async fn execute_campaign(
    cmd: CampaignCommand,
    edm: &EdmClient<'_>,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    match cmd {
        CampaignCommand::Submit { fields } => {
            let html_content =
                resolve_campaign_html(fields.html.as_deref(), fields.html_file.as_deref())?;
            warn_edm_variable_syntax(&html_content);
            warn_edm_variable_syntax(&fields.subject);

            let request = build_campaign_submit(&fields, &html_content)?;
            edm.submit_campaign(&request).await
        }

        CampaignCommand::SubmitOnce {
            contacts_file,
            name,
            subject,
            from_name,
            from_address,
            html,
            html_file,
            footer_lang,
            preheader,
            schedule,
            schedule_date,
            schedule_timezone,
            ga,
            ga_ecommerce,
            utm_campaign,
            utm_content,
        } => {
            let html_content = resolve_campaign_html(html.as_deref(), html_file.as_deref())?;
            warn_edm_variable_syntax(&html_content);
            warn_edm_variable_syntax(&subject);

            let contacts_csv = std::fs::read_to_string(&contacts_file)?;
            let footer = parse_footer_lang(&footer_lang)?;
            let sched_type = parse_schedule_type(&schedule)?;

            let request = CampaignOnceRequest {
                form: CampaignOnceForm {
                    name,
                    contacts_file_content: contacts_csv,
                    exclude_lists: vec![],
                },
                content: CampaignContent {
                    subject,
                    from_name,
                    from_address,
                    html_content,
                    footer_lang: footer,
                    preheader,
                },
                config: CampaignConfig {
                    schedule: ScheduleConfig {
                        schedule_type: sched_type,
                        timezone: schedule_timezone,
                        schedule_date,
                    },
                    ga: GaConfig {
                        enable: ga,
                        ecommerce_enable: ga_ecommerce,
                        utm_campaign,
                        utm_content,
                    },
                },
            };
            edm.submit_campaign_once(&request).await
        }

        CampaignCommand::Delete { sns } => {
            let sn_list = split_comma(&sns);
            let request = CampaignDeleteRequest { sns: sn_list };
            edm.delete_campaigns(&request).await
        }

        CampaignCommand::Pause { sn } => edm.pause_campaign(&sn).await,

        CampaignCommand::Status { sn } => edm.campaign_status(&sn).await,

        CampaignCommand::Analyze { sn } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool("analyze_campaign", serde_json::json!({"sn": sn}))
                .await
        }

        CampaignCommand::Compare { sns } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool("compare_campaigns", serde_json::json!({"sns": sns}))
                .await
        }

        CampaignCommand::Preflight { sn } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool("preflight_check_campaign", serde_json::json!({"sn": sn}))
                .await
        }

        CampaignCommand::Find { query } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool("find_campaigns", serde_json::json!({"query": query}))
                .await
        }

        CampaignCommand::BestTime => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool("get_best_send_time", serde_json::json!({}))
                .await
        }
    }
}

async fn execute_ab_test(cmd: AbTestCommand, edm: &EdmClient<'_>) -> Result<Value, NlError> {
    match cmd {
        AbTestCommand::Submit { fields } => {
            let request = build_ab_test_request(&fields, None)?;
            edm.submit_ab_test(&request).await
        }

        AbTestCommand::SubmitOnce {
            contacts_file,
            fields,
        } => {
            let contacts_csv = std::fs::read_to_string(&contacts_file)?;
            let ab_content = build_ab_test_content(&fields)?;
            let config = build_ab_test_config(&fields)?;

            let exclude_lists = fields
                .exclude_lists
                .as_deref()
                .map(split_comma)
                .unwrap_or_default();

            let request = AbTestOnceRequest {
                form: CampaignOnceForm {
                    name: fields.name.clone(),
                    contacts_file_content: contacts_csv,
                    exclude_lists,
                },
                content: ab_content,
                config,
            };
            edm.submit_ab_test_once(&request).await
        }
    }
}

async fn execute_report(
    cmd: ReportCommand,
    edm: &EdmClient<'_>,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    match cmd {
        ReportCommand::List {
            start_date,
            end_date,
        } => edm.report_list(&start_date, &end_date).await,

        ReportCommand::Metrics { sns } => {
            let sn_list = split_comma(&sns);
            edm.report_metrics(&sn_list).await
        }

        ReportCommand::Export { sn, wait, output } => {
            if wait {
                let output_path = output.ok_or_else(|| {
                    NlError::Validation(
                        "--output is required when using --wait for report export".into(),
                    )
                })?;
                helpers::report_export::execute(&sn, &output_path, edm).await
            } else {
                edm.report_export(&sn).await
            }
        }

        ReportCommand::DownloadLink { sn } => edm.report_download_link(&sn).await,

        ReportCommand::Summary { days } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool(
                "get_recent_campaigns_summary",
                serde_json::json!({"days": days}),
            )
            .await
        }

        ReportCommand::Clicks { sn } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool("get_click_details", serde_json::json!({"sn": sn}))
                .await
        }
    }
}

async fn execute_template(
    cmd: TemplateCommand,
    edm: &EdmClient<'_>,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    match cmd {
        TemplateCommand::List => edm.list_templates().await,

        TemplateCommand::Get { id, output } => {
            let result = edm.get_template(&id).await?;

            // If --output is specified, save the HTML content to the file.
            if let Some(output_path) = output {
                if let Some(html) = result["html"].as_str() {
                    std::fs::write(&output_path, html)?;
                    eprintln!("Template saved to {}", output_path.display());
                }
            }

            Ok(result)
        }

        TemplateCommand::Save { campaign_sn, name } => {
            let mut mcp = McpClient::new(client, config.edm_api_key()?, config.mcp_url());
            mcp.call_tool(
                "save_template",
                serde_json::json!({"campaign_sn": campaign_sn, "name": name}),
            )
            .await
        }
    }
}

async fn execute_automation(cmd: AutomationCommand, edm: &EdmClient<'_>) -> Result<Value, NlError> {
    match cmd {
        AutomationCommand::Trigger {
            workflow: _,
            event,
            recipients,
            recipients_file,
        } => {
            // Note: The CLI accepts --workflow for future extensibility, but the current
            // AutomationTriggerRequest API only supports event + recipients. The workflow
            // parameter is accepted but not used in the API call.
            let recipient_list = parse_automation_recipients(recipients, recipients_file)?;
            let request = AutomationTriggerRequest {
                event,
                recipients: recipient_list,
            };
            edm.trigger_automation(&request).await
        }
    }
}

async fn execute_account(cmd: AccountCommand, edm: &EdmClient<'_>) -> Result<Value, NlError> {
    match cmd {
        AccountCommand::Balance => edm.get_balance().await,
    }
}

// ── Surenotify commands ─────────────────────────────────────────────────────

async fn execute_sn(cmd: SnCommand, sn: &SurenotifyClient<'_>) -> Result<Value, NlError> {
    match cmd {
        SnCommand::Email(args) => execute_email(args.command, sn).await,
        SnCommand::Sms(args) => execute_sms(args.command, sn).await,
        SnCommand::Webhook(args) => execute_webhook(args.command, sn).await,
        SnCommand::SmsWebhook(args) => execute_sms_webhook(args.command, sn).await,
        SnCommand::Domain(args) => execute_domain(args.command, sn).await,
    }
}

async fn execute_email(cmd: EmailCommand, sn: &SurenotifyClient<'_>) -> Result<Value, NlError> {
    match cmd {
        EmailCommand::Send {
            subject,
            from_address,
            from_name,
            html,
            html_file,
            to,
            recipients,
            recipients_file,
            unsubscribe_link,
        } => {
            let content = resolve_campaign_html(html.as_deref(), html_file.as_deref())?;
            warn_sn_variable_syntax(&content);
            warn_sn_variable_syntax(&subject);

            let recipient_list = parse_email_recipients(to, recipients, recipients_file)?;

            if recipient_list.is_empty() {
                return Err(NlError::Validation(
                    "At least one recipient is required (--to, --recipients, or --recipients-file)"
                        .into(),
                ));
            }

            let request = EmailSendRequest {
                subject,
                from_address,
                content,
                recipients: recipient_list,
                from_name,
                unsubscribed_link: unsubscribe_link,
            };
            sn.send_email(&request).await
        }

        EmailCommand::Events {
            id,
            recipient,
            from,
            to_date,
            status,
            page,
            size,
        } => {
            let params = EmailEventsParams {
                id,
                recipient,
                from,
                to: to_date,
                status,
                page,
                size,
            };
            sn.email_events(&params).await
        }
    }
}

async fn execute_sms(cmd: SmsCommand, sn: &SurenotifyClient<'_>) -> Result<Value, NlError> {
    match cmd {
        SmsCommand::Send {
            content,
            phone,
            country_code,
            recipients,
            recipients_file,
            from,
            alive_mins,
        } => {
            warn_sn_variable_syntax(&content);

            let recipient_list =
                parse_sms_recipients(phone, country_code, recipients, recipients_file)?;

            if recipient_list.is_empty() {
                return Err(NlError::Validation(
                    "At least one recipient is required (--phone, --recipients, or --recipients-file)"
                        .into(),
                ));
            }

            let request = SmsSendRequest {
                content,
                recipients: recipient_list,
                from,
                alive_mins,
            };
            sn.send_sms(&request).await
        }

        SmsCommand::Events {
            id,
            recipient,
            country_code,
            from,
            to_date,
            status,
            page,
            size,
        } => {
            let params = SmsEventsParams {
                id,
                recipient,
                country_code,
                from,
                to: to_date,
                status,
                page,
                size,
            };
            sn.sms_events(&params).await
        }

        SmsCommand::ExclusiveNumber => sn.exclusive_number().await,
    }
}

async fn execute_webhook(cmd: WebhookCommand, sn: &SurenotifyClient<'_>) -> Result<Value, NlError> {
    match cmd {
        WebhookCommand::Create { event_type, url } => {
            let etype = parse_webhook_event_type(&event_type)?;
            let request = WebhookRequest {
                event_type: etype,
                url,
            };
            sn.create_webhook(&request).await
        }

        WebhookCommand::List => sn.list_webhooks().await,

        WebhookCommand::Delete { event_type } => {
            let etype = parse_webhook_event_type(&event_type)?;
            sn.delete_webhook(etype).await
        }
    }
}

async fn execute_sms_webhook(
    cmd: SmsWebhookCommand,
    sn: &SurenotifyClient<'_>,
) -> Result<Value, NlError> {
    match cmd {
        SmsWebhookCommand::Create { event_type, url } => {
            let etype = parse_sms_webhook_event_type(&event_type)?;
            let request = SmsWebhookRequest {
                event_type: etype,
                url,
            };
            sn.create_sms_webhook(&request).await
        }

        SmsWebhookCommand::List => sn.list_sms_webhooks().await,

        SmsWebhookCommand::Delete { event_type } => {
            let etype = parse_sms_webhook_event_type(&event_type)?;
            sn.delete_sms_webhook(etype).await
        }
    }
}

async fn execute_domain(cmd: DomainCommand, sn: &SurenotifyClient<'_>) -> Result<Value, NlError> {
    match cmd {
        DomainCommand::Create { domain } => sn.create_domain(&domain).await,
        DomainCommand::Verify { domain } => sn.verify_domain(&domain).await,
        DomainCommand::Remove { domain } => sn.remove_domain(&domain).await,
    }
}

// ── Helper commands ─────────────────────────────────────────────────────────

async fn execute_helper(
    cmd: HelperCommand,
    config: &ResolvedConfig,
    client: &ApiClient,
) -> Result<Value, NlError> {
    match cmd {
        HelperCommand::CampaignSend { campaign, wait } => {
            let edm = EdmClient::new(client, config.edm_api_key()?);
            helpers::campaign_send::execute(&campaign, wait, &edm).await
        }

        HelperCommand::ImportAndWait {
            list_sn,
            file,
            timeout,
            poll_interval,
        } => {
            let edm = EdmClient::new(client, config.edm_api_key()?);
            helpers::import_wait::execute(&list_sn, &file, timeout, poll_interval, &edm).await
        }

        HelperCommand::ReportDownload { sn, output } => {
            let edm = EdmClient::new(client, config.edm_api_key()?);
            helpers::report_export::execute(&sn, &output, &edm).await
        }

        HelperCommand::DomainSetup {
            domain,
            auto_verify_after,
        } => {
            let sn = SurenotifyClient::new(client, config.sn_api_key()?);
            helpers::domain_setup::execute(&domain, auto_verify_after, &sn).await
        }
    }
}

// ── Shared parsing utilities ────────────────────────────────────────────────

/// Split a comma-separated string into a Vec, trimming whitespace.
fn split_comma(s: &str) -> Vec<String> {
    s.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Resolve HTML content from either an inline string or a file path.
pub(crate) fn resolve_campaign_html(
    html: Option<&str>,
    html_file: Option<&Path>,
) -> Result<String, NlError> {
    if let Some(html) = html {
        return Ok(html.to_string());
    }
    if let Some(path) = html_file {
        return std::fs::read_to_string(path).map_err(NlError::Io);
    }
    Err(NlError::Validation(
        "Either --html or --html-file must be provided".into(),
    ))
}

/// Resolve CSV text from either an inline string or a file path.
fn resolve_csv_text(csv_text: Option<String>, csv_file: Option<&Path>) -> Result<String, NlError> {
    if let Some(text) = csv_text {
        return Ok(text);
    }
    if let Some(path) = csv_file {
        return std::fs::read_to_string(path).map_err(NlError::Io);
    }
    Err(NlError::Validation(
        "Either --csv-text or --csv-file must be provided".into(),
    ))
}

/// Parse footer language string to API numeric code.
pub(crate) fn parse_footer_lang(lang: &str) -> Result<u8, NlError> {
    match lang.to_lowercase().as_str() {
        "chinese" | "zh" => Ok(1),
        "english" | "en" => Ok(0),
        "japanese" | "ja" => Ok(2),
        _ => Err(NlError::Validation(format!(
            "Invalid footer language: '{}'. Valid: chinese, english, japanese",
            lang
        ))),
    }
}

/// Parse schedule type string to API numeric code.
fn parse_schedule_type(schedule: &str) -> Result<u8, NlError> {
    match schedule.to_lowercase().as_str() {
        "immediate" => Ok(0),
        "scheduled" => Ok(1),
        _ => Err(NlError::Validation(format!(
            "Invalid schedule type: '{}'. Valid: immediate, scheduled",
            schedule
        ))),
    }
}

/// Parse email webhook event type string to numeric code.
fn parse_webhook_event_type(event_type: &str) -> Result<u8, NlError> {
    match event_type.to_lowercase().as_str() {
        "delivery" => Ok(3),
        "open" => Ok(4),
        "click" => Ok(5),
        "bounce" => Ok(6),
        "complaint" => Ok(7),
        _ => Err(NlError::Validation(format!(
            "Invalid event type: '{}'. Valid: delivery, open, click, bounce, complaint",
            event_type
        ))),
    }
}

/// Parse SMS webhook event type string to numeric code.
fn parse_sms_webhook_event_type(event_type: &str) -> Result<u8, NlError> {
    match event_type.to_lowercase().as_str() {
        "delivery" => Ok(3),
        "bounce" => Ok(6),
        _ => Err(NlError::Validation(format!(
            "Invalid SMS event type: '{}'. Valid: delivery, bounce",
            event_type
        ))),
    }
}

/// Parse A/B test "test_on" string to numeric code.
fn parse_test_on(test_on: &str) -> Result<u8, NlError> {
    match test_on.to_lowercase().as_str() {
        "subject" => Ok(1),
        "sender" => Ok(2),
        "content" => Ok(3),
        _ => Err(NlError::Validation(format!(
            "Invalid test-on: '{}'. Valid: subject, sender, content",
            test_on
        ))),
    }
}

/// Parse test unit string to numeric code.
fn parse_test_unit(unit: &str) -> Result<u8, NlError> {
    match unit.to_lowercase().as_str() {
        "hours" | "hour" => Ok(0),
        "days" | "day" => Ok(1),
        _ => Err(NlError::Validation(format!(
            "Invalid test unit: '{}'. Valid: hours, days",
            unit
        ))),
    }
}

/// Warn if EDM content contains Surenotify-style `{{...}}` variables.
pub(crate) fn warn_edm_variable_syntax(content: &str) {
    let re = Regex::new(r"\{\{[^}]+\}\}").unwrap();
    if re.is_match(content) {
        eprintln!(
            "Warning: EDM API uses ${{FIELD}} variable syntax. \
             Detected {{{{...}}}} format (Surenotify syntax)."
        );
    }
}

/// Warn if Surenotify content contains EDM-style `${...}` variables.
fn warn_sn_variable_syntax(content: &str) {
    let re = Regex::new(r"\$\{[^}]+\}").unwrap();
    if re.is_match(content) {
        eprintln!(
            "Warning: Surenotify API uses {{{{variable}}}} variable syntax. \
             Detected ${{{{...}}}} format (EDM syntax)."
        );
    }
}

// ── Request builders ────────────────────────────────────────────────────────

/// Build a `CampaignSubmitRequest` from CLI fields.
pub(crate) fn build_campaign_submit(
    fields: &CampaignSubmitFields,
    html_content: &str,
) -> Result<CampaignSubmitRequest, NlError> {
    let selected_lists = split_comma(&fields.lists);
    if selected_lists.is_empty() {
        return Err(NlError::Validation(
            "At least one list SN is required (--lists)".into(),
        ));
    }

    let exclude_lists = fields
        .exclude_lists
        .as_deref()
        .map(split_comma)
        .unwrap_or_default();

    let footer_lang = parse_footer_lang(&fields.footer_lang)?;
    let schedule_type = parse_schedule_type(&fields.schedule)?;

    Ok(CampaignSubmitRequest {
        form: CampaignForm {
            name: fields.name.clone(),
            selected_lists,
            exclude_lists,
        },
        content: CampaignContent {
            subject: fields.subject.clone(),
            from_name: fields.from_name.clone(),
            from_address: fields.from_address.clone(),
            html_content: html_content.to_string(),
            footer_lang,
            preheader: fields.preheader.clone(),
        },
        config: CampaignConfig {
            schedule: ScheduleConfig {
                schedule_type,
                timezone: fields.schedule_timezone,
                schedule_date: fields.schedule_date.clone(),
            },
            ga: GaConfig {
                enable: fields.ga,
                ecommerce_enable: fields.ga_ecommerce,
                utm_campaign: fields.utm_campaign.clone(),
                utm_content: fields.utm_content.clone(),
            },
        },
    })
}

/// Build an `AbTestSubmitRequest` from the A/B test CLI fields.
fn build_ab_test_request(
    fields: &AbTestSubmitFields,
    _contacts_file: Option<&Path>,
) -> Result<AbTestSubmitRequest, NlError> {
    let selected_lists = split_comma(&fields.lists);
    if selected_lists.is_empty() {
        return Err(NlError::Validation(
            "At least one list SN is required (--lists)".into(),
        ));
    }

    let exclude_lists = fields
        .exclude_lists
        .as_deref()
        .map(split_comma)
        .unwrap_or_default();

    let content = build_ab_test_content(fields)?;
    let config = build_ab_test_config(fields)?;

    Ok(AbTestSubmitRequest {
        form: CampaignForm {
            name: fields.name.clone(),
            selected_lists,
            exclude_lists,
        },
        content,
        config,
    })
}

/// Build the `AbTestContent` from CLI fields.
fn build_ab_test_content(fields: &AbTestSubmitFields) -> Result<AbTestContent, NlError> {
    let testing_on = parse_test_on(&fields.test_on)?;
    let test_unit = parse_test_unit(&fields.test_unit)?;
    let footer_lang = parse_footer_lang(&fields.footer_lang)?;

    // Resolve the shared HTML content (for subject and sender tests).
    let html_content = if testing_on != 3 {
        // For subject and sender tests, use the shared --html / --html-file.
        Some(resolve_campaign_html(
            fields.html.as_deref(),
            fields.html_file.as_deref(),
        )?)
    } else {
        None
    };

    // Resolve variant HTML content (for content tests).
    let html_content_a = if testing_on == 3 {
        fields
            .html_content_a_file
            .as_ref()
            .map(|p| std::fs::read_to_string(p).map_err(NlError::Io))
            .transpose()?
    } else {
        None
    };
    let html_content_b = if testing_on == 3 {
        fields
            .html_content_b_file
            .as_ref()
            .map(|p| std::fs::read_to_string(p).map_err(NlError::Io))
            .transpose()?
    } else {
        None
    };

    Ok(AbTestContent {
        testing_on,
        testing: AbTestConfig {
            proportion: fields.proportion,
            time: fields.test_duration,
            unit: test_unit,
        },
        subject: fields.subject.clone(),
        from_name: fields.from_name.clone(),
        from_address: fields.from_address.clone(),
        html_content,
        preheader: fields.preheader.clone(),
        footer_lang: Some(footer_lang),
        subject_a: fields.subject_a.clone(),
        subject_b: fields.subject_b.clone(),
        from_name_a: fields.from_name_a.clone(),
        from_name_b: fields.from_name_b.clone(),
        from_address_a: fields.from_address_a.clone(),
        from_address_b: fields.from_address_b.clone(),
        html_content_a,
        html_content_b,
    })
}

/// Build the `CampaignConfig` from A/B test CLI fields.
fn build_ab_test_config(fields: &AbTestSubmitFields) -> Result<CampaignConfig, NlError> {
    let schedule_type = parse_schedule_type(&fields.schedule)?;

    Ok(CampaignConfig {
        schedule: ScheduleConfig {
            schedule_type,
            timezone: fields.schedule_timezone,
            schedule_date: fields.schedule_date.clone(),
        },
        ga: GaConfig {
            enable: fields.ga,
            ecommerce_enable: fields.ga_ecommerce,
            utm_campaign: fields.utm_campaign.clone(),
            utm_content: fields.utm_content.clone(),
        },
    })
}

// ── Recipient parsers ───────────────────────────────────────────────────────

/// Parse email recipients from one of three sources:
/// - `--to`: comma-separated email addresses
/// - `--recipients`: JSON array string
/// - `--recipients-file`: JSON array file
fn parse_email_recipients(
    to: Option<String>,
    recipients_json: Option<String>,
    recipients_file: Option<std::path::PathBuf>,
) -> Result<Vec<EmailRecipient>, NlError> {
    if let Some(to_str) = to {
        // Simple comma-separated addresses.
        return Ok(to_str
            .split(',')
            .map(|addr| {
                let addr = addr.trim().to_string();
                EmailRecipient {
                    name: addr.clone(),
                    address: addr,
                    variables: None,
                }
            })
            .filter(|r| !r.address.is_empty())
            .collect());
    }

    if let Some(json_str) = recipients_json {
        let recipients: Vec<EmailRecipient> = serde_json::from_str(&json_str)?;
        return Ok(recipients);
    }

    if let Some(path) = recipients_file {
        let content = std::fs::read_to_string(&path)?;
        let recipients: Vec<EmailRecipient> = serde_json::from_str(&content)?;
        return Ok(recipients);
    }

    Ok(Vec::new())
}

/// Parse SMS recipients from one of three sources:
/// - `--phone` + `--country-code`: single recipient
/// - `--recipients`: JSON array string
/// - `--recipients-file`: JSON array file
fn parse_sms_recipients(
    phone: Option<String>,
    country_code: Option<String>,
    recipients_json: Option<String>,
    recipients_file: Option<std::path::PathBuf>,
) -> Result<Vec<SmsRecipient>, NlError> {
    if let Some(phone_num) = phone {
        let cc = country_code.ok_or_else(|| {
            NlError::Validation("--country-code is required when using --phone".into())
        })?;
        return Ok(vec![SmsRecipient {
            address: phone_num,
            country_code: cc,
            variables: None,
        }]);
    }

    if let Some(json_str) = recipients_json {
        let recipients: Vec<SmsRecipient> = serde_json::from_str(&json_str)?;
        return Ok(recipients);
    }

    if let Some(path) = recipients_file {
        let content = std::fs::read_to_string(&path)?;
        let recipients: Vec<SmsRecipient> = serde_json::from_str(&content)?;
        return Ok(recipients);
    }

    Ok(Vec::new())
}

/// Parse automation recipients from `--recipients` (comma-sep emails) or `--recipients-file`.
fn parse_automation_recipients(
    recipients: Option<String>,
    recipients_file: Option<std::path::PathBuf>,
) -> Result<Vec<AutomationRecipient>, NlError> {
    if let Some(csv_str) = recipients {
        return Ok(csv_str
            .split(',')
            .map(|addr| {
                let addr = addr.trim().to_string();
                AutomationRecipient {
                    name: addr.clone(),
                    address: addr,
                    variables: None,
                }
            })
            .filter(|r| !r.address.is_empty())
            .collect());
    }

    if let Some(path) = recipients_file {
        let content = std::fs::read_to_string(&path)?;
        // Try JSON first, then fall back to line-delimited.
        if content.trim_start().starts_with('[') {
            let recipients: Vec<AutomationRecipient> = serde_json::from_str(&content)?;
            return Ok(recipients);
        }
        // Line-delimited email addresses.
        return Ok(content
            .lines()
            .map(|line| {
                let addr = line.trim().to_string();
                AutomationRecipient {
                    name: addr.clone(),
                    address: addr,
                    variables: None,
                }
            })
            .filter(|r| !r.address.is_empty())
            .collect());
    }

    Err(NlError::Validation(
        "Either --recipients or --recipients-file is required".into(),
    ))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_comma() {
        assert_eq!(split_comma("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(split_comma("a, b , c"), vec!["a", "b", "c"]);
        assert_eq!(split_comma("single"), vec!["single"]);
        assert!(split_comma("").is_empty());
        assert!(split_comma(",,,").is_empty());
    }

    #[test]
    fn test_parse_footer_lang() {
        assert_eq!(parse_footer_lang("chinese").unwrap(), 1);
        assert_eq!(parse_footer_lang("English").unwrap(), 0);
        assert_eq!(parse_footer_lang("japanese").unwrap(), 2);
        assert_eq!(parse_footer_lang("zh").unwrap(), 1);
        assert_eq!(parse_footer_lang("en").unwrap(), 0);
        assert_eq!(parse_footer_lang("ja").unwrap(), 2);
        assert!(parse_footer_lang("french").is_err());
    }

    #[test]
    fn test_parse_schedule_type() {
        assert_eq!(parse_schedule_type("immediate").unwrap(), 0);
        assert_eq!(parse_schedule_type("scheduled").unwrap(), 1);
        assert_eq!(parse_schedule_type("Immediate").unwrap(), 0);
        assert!(parse_schedule_type("delayed").is_err());
    }

    #[test]
    fn test_parse_webhook_event_type() {
        assert_eq!(parse_webhook_event_type("delivery").unwrap(), 3);
        assert_eq!(parse_webhook_event_type("open").unwrap(), 4);
        assert_eq!(parse_webhook_event_type("click").unwrap(), 5);
        assert_eq!(parse_webhook_event_type("bounce").unwrap(), 6);
        assert_eq!(parse_webhook_event_type("complaint").unwrap(), 7);
        assert_eq!(parse_webhook_event_type("Delivery").unwrap(), 3);
        assert!(parse_webhook_event_type("unknown").is_err());
    }

    #[test]
    fn test_parse_sms_webhook_event_type() {
        assert_eq!(parse_sms_webhook_event_type("delivery").unwrap(), 3);
        assert_eq!(parse_sms_webhook_event_type("bounce").unwrap(), 6);
        assert!(parse_sms_webhook_event_type("open").is_err());
    }

    #[test]
    fn test_parse_test_on() {
        assert_eq!(parse_test_on("subject").unwrap(), 1);
        assert_eq!(parse_test_on("sender").unwrap(), 2);
        assert_eq!(parse_test_on("content").unwrap(), 3);
        assert!(parse_test_on("other").is_err());
    }

    #[test]
    fn test_parse_test_unit() {
        assert_eq!(parse_test_unit("hours").unwrap(), 0);
        assert_eq!(parse_test_unit("hour").unwrap(), 0);
        assert_eq!(parse_test_unit("days").unwrap(), 1);
        assert_eq!(parse_test_unit("day").unwrap(), 1);
        assert!(parse_test_unit("minutes").is_err());
    }

    #[test]
    fn test_convert_format() {
        assert_eq!(convert_format(OutputFormat::Json), Format::Json);
        assert_eq!(convert_format(OutputFormat::Table), Format::Table);
        assert_eq!(convert_format(OutputFormat::Yaml), Format::Yaml);
        assert_eq!(convert_format(OutputFormat::Csv), Format::Csv);
    }

    #[test]
    fn test_resolve_campaign_html_inline() {
        let html = resolve_campaign_html(Some("<p>hi</p>"), None).unwrap();
        assert_eq!(html, "<p>hi</p>");
    }

    #[test]
    fn test_resolve_campaign_html_neither() {
        let err = resolve_campaign_html(None, None).unwrap_err();
        assert!(matches!(err, NlError::Validation(_)));
    }

    #[test]
    fn test_parse_email_recipients_to() {
        let recipients =
            parse_email_recipients(Some("a@b.com, c@d.com".into()), None, None).unwrap();
        assert_eq!(recipients.len(), 2);
        assert_eq!(recipients[0].address, "a@b.com");
        assert_eq!(recipients[1].address, "c@d.com");
    }

    #[test]
    fn test_parse_email_recipients_json() {
        let json = r#"[{"name":"Alice","address":"a@b.com"}]"#;
        let recipients = parse_email_recipients(None, Some(json.into()), None).unwrap();
        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].name, "Alice");
    }

    #[test]
    fn test_parse_email_recipients_empty() {
        let recipients = parse_email_recipients(None, None, None).unwrap();
        assert!(recipients.is_empty());
    }

    #[test]
    fn test_parse_sms_recipients_single() {
        let recipients =
            parse_sms_recipients(Some("912345678".into()), Some("886".into()), None, None).unwrap();
        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].address, "912345678");
        assert_eq!(recipients[0].country_code, "886");
    }

    #[test]
    fn test_parse_sms_recipients_missing_country_code() {
        let err = parse_sms_recipients(Some("912345678".into()), None, None, None).unwrap_err();
        assert!(matches!(err, NlError::Validation(_)));
    }

    #[test]
    fn test_parse_automation_recipients_csv() {
        let recipients = parse_automation_recipients(Some("a@b.com,c@d.com".into()), None).unwrap();
        assert_eq!(recipients.len(), 2);
    }

    #[test]
    fn test_parse_automation_recipients_none() {
        let err = parse_automation_recipients(None, None).unwrap_err();
        assert!(matches!(err, NlError::Validation(_)));
    }

    #[test]
    fn test_warn_edm_variable_syntax_no_match() {
        // Should not panic.
        warn_edm_variable_syntax("<p>Hello ${NAME}</p>");
    }

    #[test]
    fn test_warn_sn_variable_syntax_no_match() {
        // Should not panic.
        warn_sn_variable_syntax("<p>Hello {{name}}</p>");
    }

    #[test]
    fn test_build_campaign_submit_basic() {
        let fields = CampaignSubmitFields {
            name: "Test".into(),
            lists: "SN1,SN2".into(),
            subject: "Hello".into(),
            from_name: "Brand".into(),
            from_address: "brand@example.com".into(),
            html: Some("<p>Hi</p>".into()),
            html_file: None,
            footer_lang: "chinese".into(),
            preheader: None,
            exclude_lists: Some("EX1".into()),
            schedule: "immediate".into(),
            schedule_date: None,
            schedule_timezone: None,
            ga: true,
            ga_ecommerce: false,
            utm_campaign: Some("test".into()),
            utm_content: None,
        };

        let request = build_campaign_submit(&fields, "<p>Hi</p>").unwrap();
        assert_eq!(request.form.name, "Test");
        assert_eq!(request.form.selected_lists, vec!["SN1", "SN2"]);
        assert_eq!(request.form.exclude_lists, vec!["EX1"]);
        assert_eq!(request.content.footer_lang, 1);
        assert_eq!(request.config.schedule.schedule_type, 0);
        assert!(request.config.ga.enable);
        assert_eq!(request.config.ga.utm_campaign.as_deref(), Some("test"));
    }
}
