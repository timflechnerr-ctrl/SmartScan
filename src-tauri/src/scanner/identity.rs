use super::{hidden_powershell, ScanCategory, ScanEntry};

/// Scan identity info: HWID, Computer Name, usernames, MAC, IPs
pub fn scan() -> ScanCategory {
    let mut entries = Vec::new();

    // --- Computer Name ---
    entries.push(check_computer_name());

    // --- Username ---
    entries.push(check_username());

    // --- HWID (Machine GUID) ---
    entries.push(check_hwid());

    // --- CPU ID ---
    entries.push(check_cpu_id());

    // --- Mainboard Serial ---
    entries.push(check_mainboard_serial());

    // --- Disk Serial ---
    entries.push(check_disk_serial());

    // --- MAC Address ---
    entries.extend(check_mac_addresses());

    // --- Local IP ---
    entries.push(check_local_ip());

    // --- Public IP ---
    entries.push(check_public_ip());

    ScanCategory {
        id: "identity".to_string(),
        name: "Identity & IDs".to_string(),
        icon: "fingerprint".to_string(),
        entries,
    }
}

fn check_computer_name() -> ScanEntry {
    match std::env::var("COMPUTERNAME") {
        Ok(name) => ScanEntry::info("Computer Name", &name),
        Err(_) => ScanEntry::info("Computer Name", "Unknown"),
    }
}

fn check_username() -> ScanEntry {
    match std::env::var("USERNAME") {
        Ok(name) => ScanEntry::info("Username", &name),
        Err(_) => ScanEntry::info("Username", "Unknown"),
    }
}

fn check_hwid() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystemProduct).UUID",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::info("HWID (UUID)", "Could not determine")
            } else {
                ScanEntry::info("HWID (UUID)", &val)
            }
        }
        Err(_) => ScanEntry::info("HWID (UUID)", "Could not determine"),
    }
}

fn check_cpu_id() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_Processor).ProcessorId",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::info("CPU ID", "Could not determine")
            } else {
                ScanEntry::info("CPU ID", &val)
            }
        }
        Err(_) => ScanEntry::info("CPU ID", "Could not determine"),
    }
}

fn check_mainboard_serial() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_BaseBoard).SerialNumber",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() || val == "Default string" || val == "None" {
                ScanEntry::info("Mainboard Serial", "Not available")
            } else {
                ScanEntry::info("Mainboard Serial", &val)
            }
        }
        Err(_) => ScanEntry::info("Mainboard Serial", "Could not determine"),
    }
}

fn check_disk_serial() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_DiskDrive | Select-Object -First 1).SerialNumber",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::info("Disk Serial", "Could not determine")
            } else {
                ScanEntry::info("Disk Serial", &val)
            }
        }
        Err(_) => ScanEntry::info("Disk Serial", "Could not determine"),
    }
}

fn check_mac_addresses() -> Vec<ScanEntry> {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' } | ForEach-Object { \"$($_.Name)|$($_.MacAddress)\" }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                return vec![ScanEntry::info("MAC Address", "No active adapters found")];
            }
            val.lines()
                .map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    let name = parts.first().unwrap_or(&"Unknown");
                    let mac = parts.get(1).unwrap_or(&"Unknown");
                    ScanEntry::info(&format!("MAC ({})", name), mac)
                })
                .collect()
        }
        Err(_) => vec![ScanEntry::info("MAC Address", "Could not determine")],
    }
}

fn check_local_ip() -> ScanEntry {
    match local_ip_address::local_ip() {
        Ok(ip) => ScanEntry::info("Local IP", &ip.to_string()),
        Err(_) => ScanEntry::info("Local IP", "Could not determine"),
    }
}

fn check_public_ip() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "(Invoke-WebRequest -Uri 'https://api.ipify.org' -UseBasicParsing -TimeoutSec 5).Content"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() || val.len() > 45 {
                ScanEntry::info("Public IP", "Could not determine")
            } else {
                ScanEntry::info("Public IP", &val)
            }
        }
        Err(_) => ScanEntry::info("Public IP", "Could not determine"),
    }
}
