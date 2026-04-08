use super::hidden_powershell;
use serde::{Deserialize, Serialize};

/// Result of a toggle operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToggleResult {
    pub success: bool,
    pub message: String,
    pub needs_restart: bool,
    pub new_value: String,
    pub new_status: String,
}

/// Toggle a system setting on or off
pub fn toggle_setting(setting_id: &str, enable: bool) -> Result<ToggleResult, String> {
    match setting_id {
        "hyper_v" => toggle_hyper_v(enable),
        "memory_integrity" => toggle_memory_integrity(enable),
        "realtime_protection" => toggle_realtime_protection(enable),
        "cloud_protection" => toggle_cloud_protection(enable),
        "firewall" => toggle_firewall(enable),
        "uac" => toggle_uac(enable),
        "game_mode" => toggle_game_mode(enable),
        "xbox_game_bar" => toggle_xbox_game_bar(enable),
        "developer_mode" => toggle_developer_mode(enable),
        "test_signing" => toggle_test_signing(enable),
        _ => Err(format!("Unknown setting: {}", setting_id)),
    }
}

fn run_elevated_ps(script: &str) -> Result<String, String> {
    let output = hidden_powershell()
        .args(&[
            "-NoProfile",
            "-Command",
            &format!(
                "Start-Process powershell -Verb RunAs -Wait -WindowStyle Hidden -ArgumentList '-NoProfile -Command \"{}\"'",
                script.replace('\"', "`\"")
            ),
        ])
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stderr.is_empty() && !stderr.contains("FullyQualifiedErrorId") {
        // Some stderr is expected from UAC prompts
    }

    Ok(stdout.trim().to_string())
}

fn run_ps_direct(script: &str) -> Result<String, String> {
    let output = hidden_powershell()
        .args(&["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// --- Hyper-V ---
fn toggle_hyper_v(enable: bool) -> Result<ToggleResult, String> {
    let cmd = if enable {
        "bcdedit /set hypervisorlaunchtype auto; Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-All -NoRestart -ErrorAction SilentlyContinue"
    } else {
        "bcdedit /set hypervisorlaunchtype off; Disable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-Hypervisor -NoRestart -ErrorAction SilentlyContinue"
    };

    run_elevated_ps(cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "warning")
    } else {
        ("Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Hyper-V {}. Restart required to apply.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: true,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Memory Integrity (HVCI) ---
fn toggle_memory_integrity(enable: bool) -> Result<ToggleResult, String> {
    let val = if enable { "1" } else { "0" };
    let cmd = format!(
        "Set-ItemProperty -Path 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity' -Name 'Enabled' -Value {} -Type DWord -Force",
        val
    );

    run_elevated_ps(&cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "warning")
    } else {
        ("Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Memory Integrity {}. Restart required.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: true,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Real-time Protection ---
fn toggle_realtime_protection(enable: bool) -> Result<ToggleResult, String> {
    let cmd = format!(
        "Set-MpPreference -DisableRealtimeMonitoring ${}",
        if enable { "false" } else { "true" }
    );

    run_elevated_ps(&cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "warning")
    } else {
        ("Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Real-time Protection {}.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: false,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Cloud Protection ---
fn toggle_cloud_protection(enable: bool) -> Result<ToggleResult, String> {
    let cmd = format!(
        "Set-MpPreference -MAPSReporting {}",
        if enable { "2" } else { "0" }
    );

    run_elevated_ps(&cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "warning")
    } else {
        ("Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Cloud Protection {}.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: false,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Windows Firewall ---
fn toggle_firewall(enable: bool) -> Result<ToggleResult, String> {
    let state = if enable { "on" } else { "off" };
    let cmd = format!("netsh advfirewall set allprofiles state {}", state);

    run_elevated_ps(&cmd)?;

    let (new_value, new_status) = if enable {
        ("All Profiles Enabled", "warning")
    } else {
        ("All Profiles Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Windows Firewall {}.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: false,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- UAC ---
fn toggle_uac(enable: bool) -> Result<ToggleResult, String> {
    let val = if enable { "1" } else { "0" };
    let cmd = format!(
        "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System' -Name 'EnableLUA' -Value {} -Type DWord -Force",
        val
    );

    run_elevated_ps(&cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "warning")
    } else {
        ("Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "UAC {}. Restart required.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: true,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Game Mode ---
fn toggle_game_mode(enable: bool) -> Result<ToggleResult, String> {
    let val = if enable { "1" } else { "0" };
    let cmd = format!(
        "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\GameBar' -Name 'AutoGameModeEnabled' -Value {} -Type DWord -Force",
        val
    );

    // Game Mode is HKCU so doesn't strictly need elevation, but we use it for consistency
    run_ps_direct(&cmd)?;

    let new_value = if enable { "Enabled" } else { "Disabled" };

    Ok(ToggleResult {
        success: true,
        message: format!("Game Mode {}.", if enable { "enabled" } else { "disabled" }),
        needs_restart: false,
        new_value: new_value.to_string(),
        new_status: "info".to_string(),
    })
}

// --- Xbox Game Bar ---
fn toggle_xbox_game_bar(enable: bool) -> Result<ToggleResult, String> {
    let val = if enable { "1" } else { "0" };
    let cmd = format!(
        "Set-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\GameDVR' -Name 'AppCaptureEnabled' -Value {} -Type DWord -Force; Set-ItemProperty -Path 'HKCU:\\System\\GameConfigStore' -Name 'GameDVR_Enabled' -Value {} -Type DWord -Force",
        val, val
    );

    run_ps_direct(&cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "warning")
    } else {
        ("Disabled", "ok")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Xbox Game Bar {}.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: false,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Developer Mode ---
fn toggle_developer_mode(enable: bool) -> Result<ToggleResult, String> {
    let val = if enable { "1" } else { "0" };
    let cmd = format!(
        "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock' -Name 'AllowDevelopmentWithoutDevLicense' -Value {} -Type DWord -Force",
        val
    );

    run_elevated_ps(&cmd)?;

    let (new_value, new_status) = if enable {
        ("Enabled", "ok")
    } else {
        ("Disabled", "info")
    };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Developer Mode {}.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: false,
        new_value: new_value.to_string(),
        new_status: new_status.to_string(),
    })
}

// --- Test Signing Mode ---
fn toggle_test_signing(enable: bool) -> Result<ToggleResult, String> {
    let cmd = if enable {
        "bcdedit /set testsigning on"
    } else {
        "bcdedit /set testsigning off"
    };

    run_elevated_ps(cmd)?;

    let new_value = if enable { "Enabled" } else { "Disabled" };

    Ok(ToggleResult {
        success: true,
        message: format!(
            "Test Signing Mode {}. Restart required.",
            if enable { "enabled" } else { "disabled" }
        ),
        needs_restart: true,
        new_value: new_value.to_string(),
        new_status: "info".to_string(),
    })
}
