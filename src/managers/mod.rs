use crate::execute_external_command;
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub provider: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub score: f64,
}

lazy_static::lazy_static! {
    static ref DETAILS_CACHE: Arc<Mutex<HashMap<String, HashMap<String, String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

fn parse_alternating_lines(lines: &[&str], manager: String, query: &str) -> Vec<Package> {
    let mut res = Vec::new();
    let mut i = 0;

    while i + 1 < lines.len() {
        let first_line = lines[i];
        let second_line = lines[i + 1];

        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let package = parts[0].to_string();
            let version = parts[1].to_string();
            let description = second_line.trim().to_string();

            let package_name = package.split('/').last().unwrap_or(&package).to_string();

            let score = crate::fuzzy::fuzzy_match(query, &package_name);
            res.push(Package {
                provider: manager.clone(),
                name: package,
                version,
                description,
                score,
            });
        }

        i += 2;
    }

    res.retain(|p| p.score > 0.01);
    res.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    res
}

pub fn search_pacman(search_word: &str) -> Vec<Package> {
    if search_word.trim().is_empty() {
        return Vec::new();
    }

    let output = Command::new("pacman").args(&["-Ss", search_word]).output();

    match output {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            let manager = "pacman".to_string();
            parse_alternating_lines(&lines, manager, search_word)
        }
        _ => Vec::new(),
    }
}

pub fn search_aur(search_word: &str) -> Vec<Package> {
    if search_word.trim().is_empty() {
        return Vec::new();
    }

    let output = Command::new("yay").args(&["-Ss", search_word]).output();

    match output {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();

            let manager = "aur".to_string();
            parse_alternating_lines(&lines, manager, search_word)
        }
        _ => Vec::new(),
    }
}

pub fn details_package(package: &str, provider: &str) -> Option<HashMap<String, String>> {
    {
        let cache = DETAILS_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(package) {
            return Some(cached.clone());
        }
    }
    let pure_name = package.split('/').last().unwrap_or(package);
    let output = if provider == "pacman".to_string() {
        Command::new("pacman")
            .args(&["-Si", pure_name])
            .output()
            .ok()?
    } else if provider == "aur" {
        Command::new("yay")
            .args(&["-Si", pure_name])
            .output()
            .ok()?
    } else {
        return None;
    };

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut info = HashMap::new();

    for line in output_str.lines() {
        if let Some((key, value)) = line.split_once(" : ") {
            info.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    if info.is_empty() {
        None
    } else {
        // Cache the result
        let mut cache = DETAILS_CACHE.lock().unwrap();
        cache.insert(package.to_string(), info.clone());
        Some(info)
    }
}

pub fn pacman_installation(
    terminal: &mut DefaultTerminal,
    selected_names: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected_names.is_empty() {
        return Ok(());
    }

    let pure_names: Vec<String> = selected_names
        .iter()
        .map(|name| name.split('/').last().unwrap_or(name).to_string())
        .collect();

    let mut args: Vec<String> = vec!["pacman".to_string(), "-S".to_string()];
    args.extend(pure_names);

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    execute_external_command(terminal, "sudo", &args_ref)?;

    Ok(())
}
pub fn aur_installation(
    terminal: &mut DefaultTerminal,
    selected_names: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected_names.is_empty() {
        return Ok(());
    }

    let pure_names: Vec<String> = selected_names
        .iter()
        .map(|name| name.split('/').last().unwrap_or(name).to_string())
        .collect();

    let mut args: Vec<String> = vec!["yay".to_string(), "-S".to_string()];
    args.extend(pure_names);

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    execute_external_command(terminal, "sudo", &args_ref)?;

    Ok(())
}
