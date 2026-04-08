use super::{hidden_powershell, ScanCategory, ScanEntry};
use winreg::enums::*;
use winreg::RegKey;

/// Scan system information: OS, build, architecture, etc.
pub fn scan() -> ScanCategory {
    let mut entries = Vec::new();

    // --- Windows Version & Build ---
    entries.push(check_windows_version());

    // --- Windows Edition ---
    entries.push(check_windows_edition());

    // --- Architecture ---
    entries.push(check_architecture());

    // --- Install Date ---
    entries.push(check_install_date());

    // --- Last Boot ---
    entries.push(check_last_boot());

    // --- Windows Activation ---
    entries.push(check_activation());

    // --- Developer Mode ---
    entries.push(check_developer_mode());

    // --- .NET Framework ---
    entries.push(check_dotnet());

    // --- DirectX ---
    entries.push(check_directx());

    // --- Visual C++ Redistributables ---
    entries.extend(check_vcredist());

    ScanCategory {
        id: "system".to_string(),
        name: "System Info".to_string(),
        icon: "monitor".to_string(),
        entries,
    }
}

fn check_windows_version() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        Ok(key) => {
            let display_version: String = key.get_value("DisplayVersion").unwrap_or_default();
            let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
            let ubr: u32 = key.get_value("UBR").unwrap_or(0);

            let product = key
                .get_value::<String, _>("ProductName")
                .unwrap_or_else(|_| "Windows".to_string());

            let version_str = if display_version.is_empty() {
                format!("{} (Build {}.{})", product, build, ubr)
            } else {
                format!("{} {} (Build {}.{})", product, display_version, build, ubr)
            };
            ScanEntry::info("Windows Version", &version_str)
        }
        Err(_) => ScanEntry::info("Windows Version", "Could not determine"),
    }
}

fn check_windows_edition() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        Ok(key) => {
            let edition: String = key
                .get_value("EditionID")
                .unwrap_or_else(|_| "Unknown".to_string());
            ScanEntry::info("Windows Edition", &edition)
        }
        Err(_) => ScanEntry::info("Windows Edition", "Could not determine"),
    }
}

fn check_architecture() -> ScanEntry {
    let arch = std::env::var("PROCESSOR_ARCHITECTURE").unwrap_or_else(|_| "Unknown".to_string());
    ScanEntry::info("Architecture", &arch)
}

fn check_install_date() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_OperatingSystem).InstallDate.ToString('yyyy-MM-dd HH:mm')",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::info("Install Date", "Could not determine")
            } else {
                ScanEntry::info("Install Date", &val)
            }
        }
        Err(_) => ScanEntry::info("Install Date", "Could not determine"),
    }
}

fn check_last_boot() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToString('yyyy-MM-dd HH:mm')",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::info("Last Boot", "Could not determine")
            } else {
                ScanEntry::info("Last Boot", &val)
            }
        }
        Err(_) => ScanEntry::info("Last Boot", "Could not determine"),
    }
}

fn check_activation() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "(Get-CimInstance SoftwareLicensingProduct -Filter \"ApplicationId='55c92734-d682-4d71-983e-d6ec3f16059f' AND PartialProductKey IS NOT NULL\" | Select-Object -First 1).LicenseStatus"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            match val.as_str() {
                "1" => ScanEntry::ok("Windows Activation", "Activated"),
                "0" => ScanEntry::warning("Windows Activation", "Not activated"),
                _ => ScanEntry::info("Windows Activation", &format!("Status: {}", val)),
            }
        }
        Err(_) => ScanEntry::info("Windows Activation", "Could not determine"),
    }
}

fn check_developer_mode() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\AppModelUnlock") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("AllowDevelopmentWithoutDevLicense");
            match val {
                Ok(1) => ScanEntry::ok("Developer Mode", "Enabled"),
                _ => ScanEntry::info("Developer Mode", "Disabled"),
            }
        }
        Err(_) => ScanEntry::info("Developer Mode", "Not configured"),
    }
}

fn check_dotnet() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\NET Framework Setup\\NDP\\v4\\Full") {
        Ok(key) => {
            let release: u32 = key.get_value("Release").unwrap_or(0);
            let version = match release {
                _ if release >= 533320 => "4.8.1+",
                _ if release >= 528040 => "4.8",
                _ if release >= 461808 => "4.7.2",
                _ if release >= 461308 => "4.7.1",
                _ if release >= 460798 => "4.7",
                _ if release >= 394802 => "4.6.2",
                _ if release >= 394254 => "4.6.1",
                _ => "4.6 or earlier",
            };
            ScanEntry::info(".NET Framework", version)
        }
        Err(_) => ScanEntry::info(".NET Framework", "Not installed"),
    }
}

fn check_directx() -> ScanEntry {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    match hklm.open_subkey("SOFTWARE\\Microsoft\\DirectX") {
        Ok(key) => {
            let version: String = key.get_value("Version").unwrap_or_default();
            if version.is_empty() {
                ScanEntry::info("DirectX", "Installed (version unknown)")
            } else {
                ScanEntry::info("DirectX", &version)
            }
        }
        Err(_) => ScanEntry::info("DirectX", "Could not determine"),
    }
}

fn check_vcredist() -> Vec<ScanEntry> {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-CimInstance Win32_Product | Where-Object { $_.Name -like '*Visual C++*' } | Select-Object -ExpandProperty Name | Sort-Object -Unique"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                return vec![ScanEntry::info("Visual C++ Redistributables", "None found")];
            }
            let count = val.lines().count();
            // Show summary instead of listing all
            let years: Vec<String> = val
                .lines()
                .filter_map(|line| {
                    // Extract year like 2015, 2017, 2019, 2022
                    line.split_whitespace()
                        .find(|w| {
                            w.len() == 4 && w.parse::<u32>().map_or(false, |y| y > 2000 && y < 2030)
                        })
                        .map(|s| s.to_string())
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let mut years_sorted = years;
            years_sorted.sort();
            vec![ScanEntry::info(
                "Visual C++ Redist.",
                &format!("{} installed ({})", count, years_sorted.join(", ")),
            )]
        }
        Err(_) => vec![ScanEntry::info(
            "Visual C++ Redistributables",
            "Could not determine",
        )],
    }
}
