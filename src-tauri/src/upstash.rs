use crate::scanner::ScanResult;
use serde::Deserialize;

const UPSTASH_URL: &str = "https://rested-gorilla-82257.upstash.io";
const UPSTASH_TOKEN: &str =
    "gQAAAAAAAUFRAAIncDI3MGU2YjUxODczOTI0MWE5OTAyNzYzZGU3OWQ4MTJlY3AyODIyNTc";

/// TTL in seconds (7 days)
const SCAN_TTL: u64 = 604800;

/// Response from Upstash REST API
#[derive(Debug, Deserialize)]
struct UpstashResponse {
    result: Option<serde_json::Value>,
}

/// Generate a scan ID in format SmartScan-XXXX-XXXX-XXXX-XX
fn generate_scan_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Use time-based seed + some mixing for randomness
    let mut state = seed as u64;
    let mut chars = Vec::new();
    let charset = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // no 0/O/1/I to avoid confusion

    for _ in 0..14 {
        // Simple xorshift mixing
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let idx = (state % charset.len() as u64) as usize;
        chars.push(charset[idx] as char);
    }

    format!(
        "SmartScan-{}{}{}{}-{}{}{}{}-{}{}{}{}-{}{}",
        chars[0],
        chars[1],
        chars[2],
        chars[3],
        chars[4],
        chars[5],
        chars[6],
        chars[7],
        chars[8],
        chars[9],
        chars[10],
        chars[11],
        chars[12],
        chars[13]
    )
}

/// Upload scan result to Upstash Redis, returns the scan ID
pub fn upload_scan(scan_result: &ScanResult) -> Result<String, String> {
    let scan_id = generate_scan_id();
    let json_data = serde_json::to_string(scan_result)
        .map_err(|e| format!("Failed to serialize scan data: {}", e))?;

    let client = reqwest::blocking::Client::new();

    // Upstash REST API: SET key value EX ttl
    let url = format!("{}", UPSTASH_URL);
    let body = serde_json::json!(["SET", scan_id, json_data, "EX", SCAN_TTL]);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", UPSTASH_TOKEN))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Failed to upload scan: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("Upload failed ({}): {}", status, body));
    }

    let resp: UpstashResponse = response
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    match resp.result {
        Some(serde_json::Value::String(ref s)) if s == "OK" => Ok(scan_id),
        _ => Err("Upload failed: unexpected response".to_string()),
    }
}

/// Download scan result from Upstash Redis by scan ID
pub fn download_scan(scan_id: &str) -> Result<ScanResult, String> {
    // Validate scan ID format
    if !scan_id.starts_with("SmartScan-") {
        return Err("Invalid Scan ID format. Must start with 'SmartScan-'".to_string());
    }

    let client = reqwest::blocking::Client::new();

    let url = format!("{}", UPSTASH_URL);
    let body = serde_json::json!(["GET", scan_id]);

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", UPSTASH_TOKEN))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Failed to fetch scan: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("Fetch failed ({}): {}", status, body));
    }

    let resp: UpstashResponse = response
        .json()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    match resp.result {
        Some(serde_json::Value::String(ref json_str)) => {
            let scan_result: ScanResult = serde_json::from_str(json_str)
                .map_err(|e| format!("Failed to deserialize scan data: {}", e))?;
            Ok(scan_result)
        }
        Some(serde_json::Value::Null) | None => Err("Scan ID not found or expired.".to_string()),
        _ => Err("Unexpected response format".to_string()),
    }
}
