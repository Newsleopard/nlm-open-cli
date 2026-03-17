//! Helper workflow: import contacts from a file and poll until the import
//! completes, showing a progress spinner.

use std::path::Path;
use std::time::Instant;

use serde_json::Value;

use crate::client::edm::EdmClient;
use crate::error::NlError;

/// Execute the import-and-wait workflow:
///
/// 1. Upload the contact file to the specified list
/// 2. Poll import status with a spinner until completion or timeout
/// 3. Return the final import status
pub async fn execute(
    list_sn: &str,
    file_path: &Path,
    timeout: Option<u64>,
    poll_interval: Option<u64>,
    edm_client: &EdmClient<'_>,
) -> Result<Value, NlError> {
    // 1. Import file
    let result = edm_client.import_file(list_sn, file_path, None).await?;
    let import_sn = result["importSn"].as_str().unwrap_or("").to_string();

    if import_sn.is_empty() {
        return Err(NlError::Api {
            status: 400,
            code: None,
            message: "Import response did not contain an importSn".into(),
        });
    }

    let timeout_secs = timeout.unwrap_or(600);
    let interval = poll_interval.unwrap_or(5);
    let start = Instant::now();

    // 2. Poll status with progress indicator
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Importing contacts...");

    loop {
        if start.elapsed().as_secs() > timeout_secs {
            pb.finish_with_message("Timeout");
            return Err(NlError::Network(format!(
                "Import timed out after {}s",
                timeout_secs
            )));
        }

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        pb.tick();

        let status = edm_client.import_status(&import_sn).await?;
        let state = status["status"].as_str().unwrap_or("");

        match state {
            "COMPLETE" | "COMPLETED" => {
                pb.finish_with_message("Import complete");
                return Ok(status);
            }
            "ERROR" => {
                pb.finish_with_message("Import failed");
                return Err(NlError::Api {
                    status: 400,
                    code: None,
                    message: format!("Import {} failed with ERROR status", import_sn),
                });
            }
            _ => continue,
        }
    }
}
