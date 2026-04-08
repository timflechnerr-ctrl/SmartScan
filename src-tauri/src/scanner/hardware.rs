use super::{hidden_powershell, ScanCategory, ScanEntry};
use sysinfo::System;

/// Scan hardware information: CPU, GPU, RAM, Disk
pub fn scan() -> ScanCategory {
    let mut entries = Vec::new();
    let mut sys = System::new_all();
    sys.refresh_all();

    // --- CPU ---
    entries.push(check_cpu(&sys));
    entries.push(check_cpu_cores(&sys));

    // --- RAM ---
    entries.push(check_ram(&sys));

    // --- GPU ---
    entries.extend(check_gpu());

    // --- Disks ---
    entries.extend(check_disks());

    // --- Monitor ---
    entries.push(check_resolution());

    ScanCategory {
        id: "hardware".to_string(),
        name: "Hardware".to_string(),
        icon: "cpu".to_string(),
        entries,
    }
}

fn check_cpu(sys: &System) -> ScanEntry {
    let cpu_name = sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    ScanEntry::info("CPU", &cpu_name)
}

fn check_cpu_cores(sys: &System) -> ScanEntry {
    let physical = System::physical_core_count().unwrap_or(0);
    let logical = sys.cpus().len();
    ScanEntry::info(
        "CPU Cores",
        &format!("{} Physical / {} Logical", physical, logical),
    )
}

fn check_ram(sys: &System) -> ScanEntry {
    let total_gb = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_gb = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let available_gb = total_gb - used_gb;
    ScanEntry::info(
        "RAM",
        &format!(
            "{:.1} GB Total / {:.1} GB Available",
            total_gb, available_gb
        ),
    )
}

fn check_gpu() -> Vec<ScanEntry> {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-CimInstance Win32_VideoController | ForEach-Object { \"$($_.Name)|$($_.DriverVersion)\" }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                return vec![ScanEntry::info("GPU", "Unknown")];
            }
            val.lines()
                .enumerate()
                .map(|(i, line)| {
                    let parts: Vec<&str> = line.split('|').collect();
                    let name = parts.first().unwrap_or(&"Unknown");
                    let driver = parts.get(1).unwrap_or(&"Unknown");
                    if i == 0 {
                        ScanEntry::info("GPU", &format!("{} (Driver: {})", name, driver))
                    } else {
                        ScanEntry::info(
                            &format!("GPU {}", i + 1),
                            &format!("{} (Driver: {})", name, driver),
                        )
                    }
                })
                .collect()
        }
        Err(_) => vec![ScanEntry::info("GPU", "Could not determine")],
    }
}

fn check_disks() -> Vec<ScanEntry> {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-CimInstance Win32_LogicalDisk -Filter 'DriveType=3' | ForEach-Object { \"$($_.DeviceID)|$([math]::Round($_.Size/1GB,1))|$([math]::Round($_.FreeSpace/1GB,1))\" }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                return vec![ScanEntry::info("Disks", "Could not determine")];
            }
            val.lines()
                .map(|line| {
                    let parts: Vec<&str> = line.split('|').collect();
                    let drive = parts.first().unwrap_or(&"?:");
                    let total = parts.get(1).unwrap_or(&"?");
                    let free = parts.get(2).unwrap_or(&"?");
                    ScanEntry::info(
                        &format!("Disk {}", drive),
                        &format!("{} GB Total / {} GB Free", total, free),
                    )
                })
                .collect()
        }
        Err(_) => vec![ScanEntry::info("Disks", "Could not determine")],
    }
}

fn check_resolution() -> ScanEntry {
    let output = hidden_powershell()
        .args(["-NoProfile", "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object -First 1 | ForEach-Object { \"$($_.CurrentHorizontalResolution)x$($_.CurrentVerticalResolution)\" }"])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() || val.contains("x") == false {
                ScanEntry::info("Display Resolution", "Could not determine")
            } else {
                ScanEntry::info("Display Resolution", &val)
            }
        }
        Err(_) => ScanEntry::info("Display Resolution", "Could not determine"),
    }
}
