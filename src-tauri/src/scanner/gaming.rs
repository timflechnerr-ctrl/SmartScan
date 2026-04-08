use super::{hidden_powershell, ScanCategory, ScanEntry};
use winreg::enums::*;
use winreg::RegKey;

/// Scan gaming-related settings: Game Mode, overlays, anti-cheats, etc.
pub fn scan() -> ScanCategory {
    let mut entries = Vec::new();

    // --- Game Mode ---
    entries.push(check_game_mode());

    // --- Xbox Game Bar ---
    entries.push(check_xbox_game_bar());

    // --- Running Anti-Cheats ---
    entries.extend(check_anticheats());

    // --- VM Detection ---
    entries.push(check_vm());

    // --- Running Overlays ---
    entries.extend(check_overlays());

    ScanCategory {
        id: "gaming".to_string(),
        name: "Gaming & Software".to_string(),
        icon: "gamepad".to_string(),
        entries,
    }
}

fn check_game_mode() -> ScanEntry {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("SOFTWARE\\Microsoft\\GameBar") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("AutoGameModeEnabled");
            match val {
                Ok(1) => ScanEntry::info("Game Mode", "Enabled"),
                Ok(0) => ScanEntry::info("Game Mode", "Disabled"),
                _ => ScanEntry::info("Game Mode", "Default (Enabled)"),
            }
        }
        Err(_) => ScanEntry::info("Game Mode", "Default (Enabled)"),
    }
}

fn check_xbox_game_bar() -> ScanEntry {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.open_subkey("SOFTWARE\\Microsoft\\GameBar") {
        Ok(key) => {
            let val: Result<u32, _> = key.get_value("UseNexusForGameBarEnabled");
            match val {
                Ok(0) => ScanEntry::ok("Xbox Game Bar", "Disabled"),
                _ => ScanEntry::info("Xbox Game Bar", "Enabled"),
            }
        }
        Err(_) => ScanEntry::info("Xbox Game Bar", "Default (Enabled)"),
    }
}

fn check_anticheats() -> Vec<ScanEntry> {
    let anticheats = vec![
        ("EasyAntiCheat", vec!["EasyAntiCheat.exe", "EasyAntiCheat"]),
        ("BattlEye", vec!["BEService.exe", "BattlEye"]),
        (
            "Vanguard (Valorant)",
            vec!["vgc.exe", "vgtray.exe", "vgk.sys"],
        ),
        ("PunkBuster", vec!["PnkBstrA.exe", "PnkBstrB.exe"]),
        ("FACEIT Anti-Cheat", vec!["FACEIT.exe", "FACEITClient.exe"]),
        ("ESEA Anti-Cheat", vec!["ESEA.exe"]),
    ];

    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "Get-Process | Select-Object -ExpandProperty Name",
        ])
        .output();

    let running_processes: Vec<String> = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_lowercase())
            .collect(),
        Err(_) => Vec::new(),
    };

    let mut entries = Vec::new();
    let mut any_found = false;

    for (name, process_names) in &anticheats {
        let is_running = process_names.iter().any(|p| {
            let pname = p.to_lowercase().replace(".exe", "").replace(".sys", "");
            running_processes.iter().any(|r| r == &pname)
        });
        if is_running {
            entries.push(ScanEntry::warning(
                &format!("Anti-Cheat: {}", name),
                "Running",
            ));
            any_found = true;
        }
    }

    if !any_found {
        entries.push(ScanEntry::ok(
            "Anti-Cheat Software",
            "None detected running",
        ));
    }

    entries
}

fn check_vm() -> ScanEntry {
    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "(Get-CimInstance Win32_ComputerSystem).Model",
        ])
        .output();

    match output {
        Ok(out) => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            if val.contains("virtual")
                || val.contains("vmware")
                || val.contains("virtualbox")
                || val.contains("kvm")
                || val.contains("qemu")
                || val.contains("hyper-v")
                || val.contains("xen")
            {
                ScanEntry::error("Virtual Machine", &format!("Detected ({})", val))
            } else {
                ScanEntry::ok("Virtual Machine", "Not detected (Physical)")
            }
        }
        Err(_) => ScanEntry::info("Virtual Machine", "Could not determine"),
    }
}

fn check_overlays() -> Vec<ScanEntry> {
    let overlays = vec![
        ("Discord", vec!["Discord"]),
        ("Steam Overlay", vec!["GameOverlayUI", "steamwebhelper"]),
        ("GeForce Experience", vec!["NVIDIA Share", "nvsphelper64"]),
        ("MSI Afterburner", vec!["MSIAfterburner"]),
        ("RivaTuner (RTSS)", vec!["RTSS"]),
        ("OBS Studio", vec!["obs64", "obs32"]),
    ];

    let output = hidden_powershell()
        .args([
            "-NoProfile",
            "-Command",
            "Get-Process | Select-Object -ExpandProperty Name",
        ])
        .output();

    let running_processes: Vec<String> = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_lowercase())
            .collect(),
        Err(_) => Vec::new(),
    };

    let mut entries = Vec::new();

    for (name, process_names) in &overlays {
        let is_running = process_names.iter().any(|p| {
            let pname = p.to_lowercase();
            running_processes.iter().any(|r| r == &pname)
        });
        if is_running {
            entries.push(ScanEntry::info(&format!("Overlay: {}", name), "Running"));
        }
    }

    if entries.is_empty() {
        entries.push(ScanEntry::ok("Overlay Software", "None detected running"));
    }

    entries
}
