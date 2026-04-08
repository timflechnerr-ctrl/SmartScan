pub mod gaming;
pub mod hardware;
pub mod identity;
pub mod network;
pub mod security;
pub mod system;
pub mod toggles;

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Create a PowerShell command that runs hidden (no visible CMD window)
#[cfg(target_os = "windows")]
pub fn hidden_powershell() -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let mut cmd = Command::new("powershell");
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn hidden_powershell() -> Command {
    Command::new("powershell")
}

/// A single scan result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEntry {
    pub label: String,
    pub value: String,
    /// "ok" | "warning" | "error" | "info"
    pub status: String,
}

/// A category of scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCategory {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub entries: Vec<ScanEntry>,
}

/// Full scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub categories: Vec<ScanCategory>,
    pub total_checks: usize,
    pub issues_found: usize,
}

impl ScanEntry {
    pub fn ok(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            status: "ok".to_string(),
        }
    }
    pub fn warning(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            status: "warning".to_string(),
        }
    }
    pub fn error(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            status: "error".to_string(),
        }
    }
    pub fn info(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            status: "info".to_string(),
        }
    }
}
