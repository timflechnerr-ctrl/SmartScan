mod scanner;
mod upstash;

use scanner::{ScanCategory, ScanResult};
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

/// Run a full system scan across all categories
#[tauri::command]
async fn run_scan() -> Result<ScanResult, String> {
    // Run each scanner module
    let categories: Vec<ScanCategory> = vec![
        scanner::security::scan(),
        scanner::system::scan(),
        scanner::hardware::scan(),
        scanner::identity::scan(),
        scanner::gaming::scan(),
        scanner::network::scan(),
    ];

    let total_checks: usize = categories.iter().map(|c| c.entries.len()).sum();
    let issues_found: usize = categories
        .iter()
        .flat_map(|c| &c.entries)
        .filter(|e| e.status == "error" || e.status == "warning")
        .count();

    Ok(ScanResult {
        categories,
        total_checks,
        issues_found,
    })
}

/// Run a single category scan
#[tauri::command]
async fn run_category_scan(category_id: String) -> Result<ScanCategory, String> {
    let result = match category_id.as_str() {
        "security" => scanner::security::scan(),
        "system" => scanner::system::scan(),
        "hardware" => scanner::hardware::scan(),
        "identity" => scanner::identity::scan(),
        "gaming" => scanner::gaming::scan(),
        "network" => scanner::network::scan(),
        _ => return Err(format!("Unknown category: {}", category_id)),
    };
    Ok(result)
}

/// Upload scan results to Upstash and return a scan ID
#[tauri::command]
async fn upload_scan(scan_result: ScanResult) -> Result<String, String> {
    tokio::task::spawn_blocking(move || upstash::upload_scan(&scan_result))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
}

/// Download scan results from Upstash by scan ID
#[tauri::command]
async fn import_scan(scan_id: String) -> Result<ScanResult, String> {
    tokio::task::spawn_blocking(move || upstash::download_scan(&scan_id))
        .await
        .map_err(|e| format!("Task failed: {}", e))?
}

// --- Auto-Updater Commands ---

#[derive(serde::Serialize, Clone)]
struct UpdateInfo {
    version: String,
    date: Option<String>,
    body: Option<String>,
}

/// Check if an update is available, return info if so
#[tauri::command]
async fn check_for_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            date: update.date.map(|d| d.to_string()),
            body: update.body.clone(),
        })),
        Ok(None) => Ok(None),
        Err(e) => {
            // Don't treat network errors as hard failures
            eprintln!("Update check failed: {}", e);
            Ok(None)
        }
    }
}

/// Download and install the pending update
#[tauri::command]
async fn download_and_install_update(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?
        .ok_or_else(|| "No update available".to_string())?;

    let mut downloaded: u64 = 0;
    let mut content_length: u64 = 0;

    update
        .download_and_install(
            |chunk_len, total| {
                // On first call, total contains the content length
                if let Some(len) = total {
                    if content_length == 0 {
                        content_length = len as u64;
                        let _ = window.emit("update-progress", serde_json::json!({
                            "event": "Started",
                            "contentLength": content_length
                        }));
                    }
                }
                downloaded += chunk_len as u64;
                let _ = window.emit("update-progress", serde_json::json!({
                    "event": "Progress",
                    "downloaded": downloaded,
                    "contentLength": content_length
                }));
            },
            || {},
        )
        .await
        .map_err(|e| e.to_string())?;

    // Download finished, installing
    let _ = window.emit("update-progress", serde_json::json!({
        "event": "Finished"
    }));

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Bring existing window to front when user tries to open a second instance
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
                let _ = window.show();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            run_scan,
            run_category_scan,
            upload_scan,
            import_scan,
            check_for_update,
            download_and_install_update
        ])
        .setup(|app| {
            // Set window icon for taskbar (use the icon from bundle config)
            if let Some(icon) = app.default_window_icon().cloned() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
