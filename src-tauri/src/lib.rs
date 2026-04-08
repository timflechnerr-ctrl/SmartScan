mod scanner;
mod upstash;

use scanner::{ScanCategory, ScanResult};

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

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            import_scan
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
