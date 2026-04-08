use super::{hidden_powershell, ScanCategory, ScanEntry};
use winreg::enums::*;
use winreg::RegKey;

/// Scan security-related settings: Secure Boot, Hyper-V, Defender, etc.
pub fn scan() -> ScanCategory {
    let mut entries = Vec::new();

    // --- Secure Boot ---
    entries.push(check_secure_boot());

    // --- UEFI vs Legacy ---
    entries.push(check_uefi_mode());

    // --- TPM ---
    entries.push(check_tpm());

    // --- Hyper-V ---
    entries.push(check_hyperv());

    // --- VBS (Virtualization Based Security) ---
    entries.push(check_vbs());

    // --- Memory Integrity / HVCI ---
    entries.push(check_memory_integrity());

    // --- Windows Defender Real-time Protection ---
    entries.push(check_realtime_protection());

    // --- Tamper Protection ---
    entries.push(check_tamper_protection());

    // --- Cloud Protection ---
    entries.push(check_cloud_protection());

    // --- Firewall ---
    entries.push(check_firewall());

    // --- UAC ---
    entries.push(check_uac());

    // --- Test Signing Mode ---
    entries.push(check_test_signing());

    // --- Driver Signature Enforcement ---
    entries.push(check_driver_signature_enforcement());

    // --- SmartScreen ---
    entries.push(check_smartscreen());

    ScanCategory {
        id: "security".to_string(),
        name: "Security & Protection".to_string(),
        icon: "shield".to_string(),
        entries,
    }
}

fn check_secure_boot() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command", "Confirm-SecureBootUEFI"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if stdout == "True" {
                ScanEntry::warning("Secure Boot", "Enabled")
            } else if stdout == "False" {
                ScanEntry::ok("Secure Boot", "Disabled")
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                if stderr.contains("not supported") || stderr.contains("Cmdlet not supported") {
                    ScanEntry::info("Secure Boot", "Not supported (Legacy BIOS)")
                } else {
                    ScanEntry::info("Secure Boot", "Could not determine")
                }
            }
        }
        Err(_) => ScanEntry::info("Secure Boot", "Could not determine"),
    }
}

fn check_uefi_mode() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "if ($env:firmware_type) { $env:firmware_type } else { 'Unknown' }",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() || val == "Unknown" {
                // Fallback: check bcdedit
                let bcdedit = hidden_powershell()
                    .args([
                        "-NoProfile",
                        "-Command",
                        "bcdedit | Select-String 'path' | Select-Object -First 1",
                    ])
                    .output();
                match bcdedit {
                    Ok(b) => {
                        let bval = String::from_utf8_lossy(&b.stdout).trim().to_string();
                        if bval.contains(".efi") {
                            ScanEntry::info("Boot Mode", "UEFI")
                        } else {
                            ScanEntry::info("Boot Mode", "Legacy BIOS")
                        }
                    }
                    Err(_) => ScanEntry::info("Boot Mode", "Unknown"),
                }
            } else {
                ScanEntry::info("Boot Mode", &val)
            }
        }
        Err(_) => ScanEntry::info("Boot Mode", "Unknown"),
    }
}

fn check_tpm() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "Get-Tpm | Select-Object -ExpandProperty TpmPresent",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val == "True" {
                // Get version
                let ver_out = hidden_powershell()
                    .args(["-NoProfile", "-Command",
                        "(Get-WmiObject -Namespace 'root\\cimv2\\Security\\MicrosoftTpm' -Class Win32_Tpm).SpecVersion"])
                    .output();
                let version = match ver_out {
                    Ok(v) => {
                        let vs = String::from_utf8_lossy(&v.stdout).trim().to_string();
                        if vs.is_empty() {
                            "Present".to_string()
                        } else {
                            format!("Version {}", vs.split(',').next().unwrap_or(&vs))
                        }
                    }
                    Err(_) => "Present".to_string(),
                };
                ScanEntry::info("TPM", &version)
            } else {
                ScanEntry::info("TPM", "Not present")
            }
        }
        Err(_) => ScanEntry::info("TPM", "Could not determine"),
    }
}

fn check_hyperv() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "(Get-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V-All -ErrorAction SilentlyContinue).State"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val == "Enabled" {
                ScanEntry::warning("Hyper-V", "Enabled")
            } else if val == "Disabled" || val.is_empty() {
                ScanEntry::ok("Hyper-V", "Disabled")
            } else {
                ScanEntry::info("Hyper-V", &val)
            }
        }
        Err(_) => ScanEntry::info("Hyper-V", "Could not determine"),
    }
}

fn check_vbs() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\DeviceGuard") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("EnableVirtualizationBasedSecurity");
            match val {
                Ok(1) => ScanEntry::warning("VBS (Virtualization Based Security)", "Enabled"),
                Ok(0) => ScanEntry::ok("VBS (Virtualization Based Security)", "Disabled"),
                _ => ScanEntry::info("VBS (Virtualization Based Security)", "Not configured"),
            }
        }
        Err(_) => ScanEntry::info("VBS (Virtualization Based Security)", "Not configured"),
    }
}

fn check_memory_integrity() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SYSTEM\\CurrentControlSet\\Control\\DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("Enabled");
            match val {
                Ok(1) => ScanEntry::warning("Memory Integrity (HVCI)", "Enabled"),
                Ok(0) => ScanEntry::ok("Memory Integrity (HVCI)", "Disabled"),
                _ => ScanEntry::ok("Memory Integrity (HVCI)", "Not configured"),
            }
        }
        Err(_) => ScanEntry::ok("Memory Integrity (HVCI)", "Not configured"),
    }
}

fn check_realtime_protection() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows Defender\\Real-Time Protection") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("DisableRealtimeMonitoring");
            match val {
                Ok(1) => ScanEntry::ok("Real-time Protection", "Disabled"),
                Ok(0) => ScanEntry::warning("Real-time Protection", "Enabled"),
                _ => ScanEntry::warning("Real-time Protection", "Enabled (default)"),
            }
        }
        Err(_) => {
            // Fallback via PowerShell
            let output = hidden_powershell()
                .args([
                    "-NoProfile",
                    "-Command",
                    "(Get-MpPreference).DisableRealtimeMonitoring",
                ])
                .output();
            match output {
                Ok(out) => {
                    let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if val == "True" {
                        ScanEntry::ok("Real-time Protection", "Disabled")
                    } else {
                        ScanEntry::warning("Real-time Protection", "Enabled")
                    }
                }
                Err(_) => ScanEntry::info("Real-time Protection", "Could not determine"),
            }
        }
    }
}

fn check_tamper_protection() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows Defender\\Features") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("TamperProtection");
            match val {
                Ok(5) => ScanEntry::warning("Tamper Protection", "Enabled"),
                Ok(4) | Ok(0) => ScanEntry::ok("Tamper Protection", "Disabled"),
                Ok(v) => ScanEntry::info("Tamper Protection", &format!("State: {}", v)),
                _ => ScanEntry::info("Tamper Protection", "Could not determine"),
            }
        }
        Err(_) => ScanEntry::info("Tamper Protection", "Could not determine"),
    }
}

fn check_cloud_protection() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows Defender\\Spynet") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("SpynetReporting");
            match val {
                Ok(0) => ScanEntry::ok("Cloud Protection", "Disabled"),
                Ok(1) => ScanEntry::info("Cloud Protection", "Basic"),
                Ok(2) => ScanEntry::warning("Cloud Protection", "Advanced"),
                _ => ScanEntry::info("Cloud Protection", "Default"),
            }
        }
        Err(_) => ScanEntry::info("Cloud Protection", "Default"),
    }
}

fn check_firewall() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "$fw = Get-NetFirewallProfile | Select-Object Name, Enabled; $fw | ForEach-Object { \"$($_.Name):$($_.Enabled)\" }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                return ScanEntry::info("Windows Firewall", "Could not determine");
            }
            let all_enabled = val.lines().all(|l| l.contains("True"));
            let all_disabled = val.lines().all(|l| l.contains("False"));
            let profiles: Vec<&str> = val.lines().collect();
            let detail = profiles.join(", ");
            if all_disabled {
                ScanEntry::ok("Windows Firewall", "All Profiles Disabled")
            } else if all_enabled {
                ScanEntry::info("Windows Firewall", "All Profiles Enabled")
            } else {
                ScanEntry::warning("Windows Firewall", &format!("Mixed ({})", detail))
            }
        }
        Err(_) => ScanEntry::info("Windows Firewall", "Could not determine"),
    }
}

fn check_uac() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("EnableLUA");
            match val {
                Ok(1) => ScanEntry::info("UAC (User Account Control)", "Enabled"),
                Ok(0) => ScanEntry::ok("UAC (User Account Control)", "Disabled"),
                _ => ScanEntry::info("UAC (User Account Control)", "Default (Enabled)"),
            }
        }
        Err(_) => ScanEntry::info("UAC (User Account Control)", "Default (Enabled)"),
    }
}

fn check_test_signing() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "bcdedit /enum | Select-String 'testsigning'",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            if val.contains("yes") {
                ScanEntry::ok("Test Signing Mode", "Enabled")
            } else {
                ScanEntry::info("Test Signing Mode", "Disabled")
            }
        }
        Err(_) => ScanEntry::info("Test Signing Mode", "Could not determine"),
    }
}

fn check_driver_signature_enforcement() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "bcdedit /enum | Select-String 'nointegritychecks'",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            if val.contains("yes") {
                ScanEntry::ok("Driver Signature Enforcement", "Disabled")
            } else {
                ScanEntry::warning("Driver Signature Enforcement", "Enabled")
            }
        }
        Err(_) => ScanEntry::info("Driver Signature Enforcement", "Could not determine"),
    }
}

fn check_smartscreen() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer") {
        Ok(key) => {
            let val: Result<String, _> = key.get_value("SmartScreenEnabled");
            match val {
                Ok(v) => {
                    if v == "Off" {
                        ScanEntry::ok("SmartScreen", "Disabled")
                    } else {
                        ScanEntry::info("SmartScreen", &format!("Enabled ({})", v))
                    }
                }
                _ => ScanEntry::info("SmartScreen", "Default (Enabled)"),
            }
        }
        Err(_) => ScanEntry::info("SmartScreen", "Default (Enabled)"),
    }
}
