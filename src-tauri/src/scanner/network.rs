use super::{hidden_powershell, ScanCategory, ScanEntry};

/// Scan network-related information: VPN, DNS, adapters
pub fn scan() -> ScanCategory {
    let mut entries = Vec::new();

    // --- Active Network Adapters ---
    entries.extend(check_adapters());

    // --- VPN Detection ---
    entries.push(check_vpn());

    // --- DNS Servers ---
    entries.push(check_dns());

    ScanCategory {
        id: "network".to_string(),
        name: "Network".to_string(),
        icon: "wifi".to_string(),
        entries,
    }
}

fn check_adapters() -> Vec<ScanEntry> {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' } | ForEach-Object { \"$($_.Name)|$($_.InterfaceDescription)|$($_.LinkSpeed)\" }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                return vec![ScanEntry::info("Network Adapters", "No active adapters")];
            }
            val.lines()
                .map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    let name = parts.first().unwrap_or(&"Unknown");
                    let desc = parts.get(1).unwrap_or(&"");
                    let speed = parts.get(2).unwrap_or(&"");
                    ScanEntry::info(
                        &format!("Adapter: {}", name),
                        &format!("{} ({})", desc, speed),
                    )
                })
                .collect()
        }
        Err(_) => vec![ScanEntry::info("Network Adapters", "Could not determine")],
    }
}

fn check_vpn() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and ($_.InterfaceDescription -like '*VPN*' -or $_.InterfaceDescription -like '*TAP*' -or $_.InterfaceDescription -like '*TUN*' -or $_.InterfaceDescription -like '*WireGuard*' -or $_.Name -like '*VPN*') } | Select-Object -ExpandProperty Name"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::ok("VPN", "Not detected")
            } else {
                ScanEntry::warning("VPN", &format!("Active ({})", val))
            }
        }
        Err(_) => ScanEntry::info("VPN", "Could not determine"),
    }
}

fn check_dns() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-DnsClientServerAddress -AddressFamily IPv4 | Where-Object { $_.ServerAddresses.Count -gt 0 } | Select-Object -First 1 | ForEach-Object { $_.ServerAddresses -join ', ' }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                ScanEntry::info("DNS Servers", "Could not determine")
            } else {
                ScanEntry::info("DNS Servers", &val)
            }
        }
        Err(_) => ScanEntry::info("DNS Servers", "Could not determine"),
    }
}
