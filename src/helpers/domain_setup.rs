//! Helper workflow: set up a sender domain by creating DNS verification
//! records and optionally auto-verifying after a wait period.

use serde_json::Value;

use crate::client::surenotify::SurenotifyClient;
use crate::error::NlError;

/// Execute the domain-setup workflow:
///
/// 1. Create domain verification records via Surenotify API
/// 2. Display the DNS records that need to be configured
/// 3. Optionally wait and then trigger automatic verification
pub async fn execute(
    domain: &str,
    auto_verify_after: Option<u64>,
    sn_client: &SurenotifyClient<'_>,
) -> Result<Value, NlError> {
    // 1. Create domain
    let dns_records = sn_client.create_domain(domain).await?;

    // 2. Display DNS records to stderr for user guidance
    eprintln!("DNS records to configure for {}:", domain);
    if let Some(records) = dns_records.as_array() {
        for r in records {
            let rtype = match r["record_type"].as_u64() {
                Some(0) => "TXT",
                Some(1) => "CNAME",
                _ => "UNKNOWN",
            };
            eprintln!(
                "  {} {} -> {}",
                rtype,
                r["name"].as_str().unwrap_or(""),
                r["value"].as_str().unwrap_or("")
            );
        }
    }

    // 3. Optional: wait and verify
    if let Some(wait_secs) = auto_verify_after {
        eprintln!("Waiting {}s before verification...", wait_secs);

        let pb = indicatif::ProgressBar::new_spinner();
        pb.set_message(format!("Waiting {}s for DNS propagation...", wait_secs));

        tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
        pb.finish_with_message("Verifying domain...");

        let verify_result = sn_client.verify_domain(domain).await?;
        return Ok(verify_result);
    }

    Ok(dns_records)
}
