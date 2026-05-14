use crate::execute_external_command;
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};

use super::Package;

fn normalize_aur_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(a) => a
            .iter()
            .map(|x| x.as_str().unwrap_or(&x.to_string()).to_string())
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Null => "None".into(),
        _ => v.to_string(),
    }
}

pub fn search_aur(search_word: &str, _aur_helper: &str) -> Vec<Package> {
    if search_word.trim().len() < 2 {
        return Vec::new();
    }

    let url = format!(
        "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
        urlencoding::encode(search_word)
    );

    let resp = match reqwest::blocking::get(url) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match resp.json() {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    if let Some(arr) = json["results"].as_array() {
        for item in arr {
            let name = item["Name"].as_str().unwrap_or("").to_string();
            let version = item["Version"].as_str().unwrap_or("").to_string();
            let description = item["Description"].as_str().unwrap_or("").to_string();
            
            let score = crate::fuzzy::fuzzy_match(search_word, &name);
            
            results.push(Package {
                provider: "aur".to_string(),
                name,
                version,
                description,
                score,
            });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

pub fn aur_details(pkg: &str) -> Option<HashMap<String, String>> {
    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg={}", pkg);

    let resp = reqwest::blocking::get(url).ok()?;
    let json: serde_json::Value = resp.json().ok()?;

    let obj = if json["results"].is_array() {
        json["results"][0].as_object()?
    } else {
        json["results"].as_object()?
    };

    let mut out = HashMap::new();

    let mapping = [
        ("Name", "Name"),
        ("Version", "Version"),
        ("Description", "Description"),
        ("URL", "URL"),
        ("License", "Licenses"),
        ("Depends", "Depends On"),
        ("OptDepends", "Optional Deps"),
        ("MakeDepends", "Make Deps"),
        ("Conflicts", "Conflicts With"),
        ("Provides", "Provides"),
        ("Replaces", "Replaces"),
        ("Keywords", "Keywords"),
        ("Submitter", "Submitter"),
        ("Maintainer", "Maintainer"),
        ("Popularity", "Popularity"),
        ("NumVotes", "Votes"),
        ("FirstSubmitted", "First Submitted"),
        ("LastModified", "Last Modified"),
    ];

    for (aur_key, pacman_key) in mapping {
        if let Some(v) = obj.get(aur_key) {
            out.insert(pacman_key.into(), normalize_aur_value(v));
        }
    }

    Some(out)
}

pub fn aur_install(
    terminal: &mut DefaultTerminal,
    selected: &HashSet<String>,
    aur_helper: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected.is_empty() {
        return Ok(());
    }

    let pure: Vec<String> = selected
        .iter()
        .map(|n| n.split('/').next_back().unwrap_or(n).to_string())
        .collect();

    let mut args: Vec<String> = vec!["-S".into(), "--needed".into()];
    args.extend(pure);

    let args_ref: Vec<&str> = args.iter().map(|x| x.as_str()).collect();
    execute_external_command(terminal, aur_helper, &args_ref)?;

    Ok(())
}

pub fn aur_upgrade(
    terminal: &mut DefaultTerminal,
    aur_helper: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    execute_external_command(terminal, aur_helper, &["-Syu"])?;
    Ok(())
}
