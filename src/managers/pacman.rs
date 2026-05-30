use super::Package;
use crate::execute_external_command;
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};

pub fn search_pacman(search_word: &str) -> Vec<Package> {
    // We'll use pacman -Ss for searching as it's more reliable than parsing .db files manually
    // and handles all configured repositories automatically.
    let output = std::process::Command::new("pacman").arg("-Ss").arg(search_word).output().ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();
        crate::managers::parse_alternating_lines(&lines, "pacman".into(), search_word)
    } else {
        Vec::new()
    }
}

pub fn pacman_info(pkg: &str) -> Option<HashMap<String, String>> {
    // Try -Si first (remote), then -Qi (local)
    let output = std::process::Command::new("pacman").arg("-Si").arg(pkg).output().ok();

    let output = if let Some(o) = output {
        if o.status.success() {
            Some(o)
        } else {
            std::process::Command::new("pacman").arg("-Qi").arg(pkg).output().ok()
        }
    } else {
        std::process::Command::new("pacman").arg("-Qi").arg(pkg).output().ok()
    };

    if let Some(output) = output {
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut out = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                if !key.is_empty() {
                    out.insert(key, value);
                }
            }
        }

        if out.is_empty() { None } else { Some(out) }
    } else {
        None
    }
}

pub fn pacman_install(
    terminal: &mut DefaultTerminal,
    selected: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected.is_empty() {
        return Ok(());
    }

    let pure: Vec<String> =
        selected.iter().map(|n| n.split('/').next_back().unwrap_or(n).to_string()).collect();

    let mut args: Vec<String> = vec!["-S".into(), "--needed".into()];
    args.extend(pure);

    let args_ref: Vec<&str> = args.iter().map(|x| x.as_str()).collect();
    execute_external_command(
        terminal,
        "sudo",
        {
            let mut full_args = vec!["pacman"];
            full_args.extend(args_ref);
            full_args
        }
        .as_slice(),
    )?;

    Ok(())
}

pub fn pacman_remove(
    terminal: &mut DefaultTerminal,
    selected: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected.is_empty() {
        return Ok(());
    }

    let pure: Vec<String> =
        selected.iter().map(|n| n.split('/').next_back().unwrap_or(n).to_string()).collect();

    let mut args: Vec<String> = vec!["-Rs".into()];
    args.extend(pure);

    let args_ref: Vec<&str> = args.iter().map(|x| x.as_str()).collect();
    execute_external_command(
        terminal,
        "sudo",
        {
            let mut full_args = vec!["pacman"];
            full_args.extend(args_ref);
            full_args
        }
        .as_slice(),
    )?;

    Ok(())
}

pub fn system_upgrade(terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    execute_external_command(terminal, "sudo", &["pacman", "-Syu"])?;
    Ok(())
}

pub fn refresh_databases(terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    execute_external_command(terminal, "sudo", &["pacman", "-Sy"])?;
    Ok(())
}

pub fn get_installed_packages() -> HashSet<String> {
    let output = std::process::Command::new("pacman").arg("-Q").output().ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next().map(|s| s.to_string()))
            .collect()
    } else {
        HashSet::new()
    }
}

pub fn get_installed_packages_details() -> Vec<Package> {
    let output = std::process::Command::new("pacman").arg("-Q").output().ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(Package {
                        provider: "pacman/local".to_string(),
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        description: "".to_string(), // we don't have desc in -Q
                        score: 1.0,
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

pub fn get_updates() -> Vec<Package> {
    let output = std::process::Command::new("checkupdates").output().ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .filter_map(|line| {
                // format: pkgname oldver -> newver
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    Some(Package {
                        provider: "pacman/update".to_string(),
                        name: parts[0].to_string(),
                        version: format!("{} -> {}", parts[1], parts[3]),
                        description: "".to_string(),
                        score: 1.0,
                    })
                } else {
                    None
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}
