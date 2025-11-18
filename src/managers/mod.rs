use crate::fuzzy::fuzzy_get_indexes;
use crate::fuzzy::score;

use std::collections::{HashMap, HashSet};
use std::process::Command;

use ratatui::DefaultTerminal;

use crate::execute_external_command;
#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub score: f64,
}

fn parse_alternating_lines(lines: &[&str], query: String) -> Vec<Package> {
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

            let clean_query = query.split_whitespace().next().unwrap_or("").to_string();

            let package_name = package.split('/').last().unwrap_or("").to_string();
            let indexes = fuzzy_get_indexes(&clean_query, &package_name);
            let score = score(clean_query.clone(), &package_name, indexes);
            //            println!("{:?}", score);
            res.push(Package {
                name: package,
                version,
                description,
                score,
            });
        }

        i += 2;
    }

    // sort in-place
    res.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    res
}
pub fn search_pacman(search_word: &str) -> Vec<Package> {
    let cmd = format!("pacman -Ss {}", &search_word);
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute pacman -Ss");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    parse_alternating_lines(&lines, search_word.to_string())
}

pub fn search_aur(search_word: &str) -> Vec<Package> {
    let cmd = format!("yay -Ss {}", &search_word);
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute pacman -Ss");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    parse_alternating_lines(&lines, search_word.to_string())
}

/*
pub fn list_installed_vec() -> Vec<Package> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("pacman -Qe")
        .output()
        .expect("Failed to execute pacman -Qe");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    parse_alternating_lines(&lines)
}*/

// this gives details, return value is not yet defined, maybe hashmap
pub fn details_package(package: &str) -> Option<HashMap<String, String>> {
    let cmd = format!("pacman -Si {}", package);
    let output = Command::new("sh").arg("-c").arg(cmd).output().ok()?; // Return None if command fails

    let output_str = String::from_utf8_lossy(&output.stdout);

    let mut info = HashMap::new();

    for line in output_str.lines() {
        if let Some((key, value)) = line.split_once(" : ") {
            info.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    if info.is_empty() { None } else { Some(info) }
}

//pacman installer
pub fn pacman_installation(
    terminal: &mut DefaultTerminal,
    selected_names: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !selected_names.is_empty() {
        let args: Vec<String> = std::iter::once("pacman".to_string())
            .chain(std::iter::once("-S".to_string()))
            .chain(selected_names.iter().cloned())
            .collect();

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        execute_external_command(terminal, "sudo", &args_ref)?;
    }

    Ok(())
}
