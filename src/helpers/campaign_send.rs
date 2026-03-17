//! Helper workflow: submit a campaign, optionally wait for completion,
//! and return final metrics.

use std::time::Instant;

use serde_json::Value;

use crate::cli::CampaignSubmitFields;
use crate::client::edm::EdmClient;
use crate::error::NlError;
use crate::executor;

/// Execute the campaign-send workflow:
///
/// 1. Check account balance
/// 2. Build and submit the campaign
/// 3. Optionally poll until the campaign finishes sending (with timeout)
/// 4. Return final metrics (or just the submit response if `!wait`)
pub async fn execute(
    fields: &CampaignSubmitFields,
    wait: bool,
    edm_client: &EdmClient<'_>,
) -> Result<Value, NlError> {
    // 1. Check balance
    let balance = edm_client.get_balance().await?;
    let email_balance = balance["email"].as_u64().unwrap_or(0);
    if email_balance == 0 {
        return Err(NlError::Validation(
            "Insufficient email balance. Check your account at https://app.newsleopard.com".into(),
        ));
    }

    // 2. Build and submit campaign
    let html_content =
        executor::resolve_campaign_html(fields.html.as_deref(), fields.html_file.as_deref())?;
    executor::warn_edm_variable_syntax(&html_content);
    executor::warn_edm_variable_syntax(&fields.subject);

    let request = executor::build_campaign_submit(fields, &html_content)?;
    let result = edm_client.submit_campaign(&request).await?;
    let campaign_sn = result["sn"].as_str().unwrap_or("").to_string();

    if !wait || campaign_sn.is_empty() {
        return Ok(result);
    }

    let timeout_secs = 600u64;
    let start = Instant::now();

    // 3. Poll status until complete
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Sending campaign...");

    loop {
        if start.elapsed().as_secs() > timeout_secs {
            pb.finish_with_message("Timeout");
            return Err(NlError::Network(format!(
                "Campaign send polling timed out after {}s",
                timeout_secs
            )));
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        pb.tick();

        let status = edm_client.campaign_status(&campaign_sn).await?;
        let state = status["status"].as_str().unwrap_or("");

        match state {
            "COMPLETE" | "SENT" | "STOP" => {
                pb.finish_with_message(format!("Campaign {}", state.to_lowercase()));

                // 4. Fetch final metrics
                let metrics = edm_client
                    .report_metrics(std::slice::from_ref(&campaign_sn))
                    .await?;
                return Ok(metrics);
            }
            "ERROR" => {
                pb.finish_with_message("Campaign failed");
                return Err(NlError::Api {
                    status: 400,
                    code: None,
                    message: format!("Campaign {} failed with ERROR status", campaign_sn),
                });
            }
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_fields(html: Option<String>, html_file: Option<PathBuf>) -> CampaignSubmitFields {
        CampaignSubmitFields {
            name: "Test Campaign".into(),
            lists: "SN1,SN2".into(),
            subject: "Hello World".into(),
            from_name: "Brand".into(),
            from_address: "brand@example.com".into(),
            html,
            html_file,
            footer_lang: "chinese".into(),
            preheader: None,
            exclude_lists: None,
            schedule: "immediate".into(),
            schedule_date: None,
            schedule_timezone: None,
            ga: false,
            ga_ecommerce: false,
            utm_campaign: None,
            utm_content: None,
        }
    }

    #[test]
    fn test_resolve_html_inline() {
        let fields = make_fields(Some("<p>Hello</p>".into()), None);
        let html =
            executor::resolve_campaign_html(fields.html.as_deref(), fields.html_file.as_deref())
                .unwrap();
        assert_eq!(html, "<p>Hello</p>");
    }

    #[test]
    fn test_resolve_html_missing() {
        let fields = make_fields(None, None);
        let err =
            executor::resolve_campaign_html(fields.html.as_deref(), fields.html_file.as_deref())
                .unwrap_err();
        assert!(matches!(err, NlError::Validation(_)));
    }

    #[test]
    fn test_parse_footer_lang() {
        assert_eq!(executor::parse_footer_lang("chinese").unwrap(), 1);
        assert_eq!(executor::parse_footer_lang("english").unwrap(), 0);
        assert_eq!(executor::parse_footer_lang("Japanese").unwrap(), 2);
        assert!(executor::parse_footer_lang("unknown").is_err());
    }

    #[test]
    fn test_build_campaign_request() {
        let fields = make_fields(Some("<p>Hi</p>".into()), None);
        let request = executor::build_campaign_submit(&fields, "<p>Hi</p>").unwrap();
        assert_eq!(request.form.name, "Test Campaign");
        assert_eq!(request.form.selected_lists, vec!["SN1", "SN2"]);
        assert!(request.form.exclude_lists.is_empty());
        assert_eq!(request.content.footer_lang, 1);
        assert_eq!(request.config.schedule.schedule_type, 0);
    }

    #[test]
    fn test_build_campaign_request_empty_lists() {
        let mut fields = make_fields(Some("<p>Hi</p>".into()), None);
        fields.lists = "".into();
        let err = executor::build_campaign_submit(&fields, "<p>Hi</p>").unwrap_err();
        assert!(matches!(err, NlError::Validation(_)));
    }

    #[test]
    fn test_warn_edm_variable_syntax_no_match() {
        // Should not panic; output goes to stderr.
        executor::warn_edm_variable_syntax("<p>Hello ${NAME}</p>");
    }

    #[test]
    fn test_warn_edm_variable_syntax_match() {
        // Should produce a warning on stderr (not testable for output, but ensures no panic).
        executor::warn_edm_variable_syntax("<p>Hello {{name}}</p>");
    }
}
