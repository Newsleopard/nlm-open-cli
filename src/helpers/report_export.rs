//! Helper workflow: trigger a campaign report export, poll for the download
//! link, and download the file.

use std::path::Path;
use std::time::Instant;

use serde_json::Value;

use crate::client::edm::EdmClient;
use crate::error::NlError;

/// Execute the report-export-and-download workflow:
///
/// 1. Trigger report export for the campaign
/// 2. Poll for the download link with a spinner
/// 3. Download the report file to the specified output path
/// 4. Return a summary JSON with status, path, and file size
pub async fn execute(
    campaign_sn: &str,
    output: &Path,
    edm_client: &EdmClient<'_>,
) -> Result<Value, NlError> {
    // 1. Trigger export
    edm_client.report_export(campaign_sn).await?;

    let timeout_secs = 600u64;
    let start = Instant::now();

    // 2. Poll for download link
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Exporting report...");

    loop {
        if start.elapsed().as_secs() > timeout_secs {
            pb.finish_with_message("Timeout");
            return Err(NlError::Network(format!(
                "Report export timed out after {}s",
                timeout_secs
            )));
        }

        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        pb.tick();

        let link_resp = edm_client.report_download_link(campaign_sn).await?;
        if let Some(link) = link_resp["link"].as_str() {
            if !link.is_empty() {
                pb.finish_with_message("Report ready");

                // 3. Download file
                let response = reqwest::get(link)
                    .await
                    .map_err(|e| NlError::Network(e.to_string()))?;
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| NlError::Network(e.to_string()))?;
                std::fs::write(output, &bytes)?;

                // 4. Return summary
                return Ok(serde_json::json!({
                    "status": "downloaded",
                    "path": output.display().to_string(),
                    "size": bytes.len()
                }));
            }
        }
    }
}
