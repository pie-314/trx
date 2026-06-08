use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn get_history_path() -> Option<PathBuf> {
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME")
        && !state_home.is_empty() {
            return Some(PathBuf::from(state_home).join("trx").join("history.log"));
        }

    if let Some(base_dirs) = directories::BaseDirs::new() {
        return Some(
            base_dirs.home_dir().join(".local").join("state").join("trx").join("history.log"),
        );
    }

    None
}

pub fn append_entry(action: &str, name: &str, version: &str, manager: &str) {
    let Some(path) = get_history_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let timestamp = Utc::now().to_rfc3339();
    // Use format required: <UTC ISO8601 timestamp>  <ACTION>  <package>  <version>  [<manager>]
    // Padding to keep it somewhat aligned, though length varies.
    let entry = format!("{}  {:<8} {:<20} {:<15} [{}]", timestamp, action, name, version, manager);

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", entry);
    }
}

pub fn read_entries() -> Vec<String> {
    let Some(path) = get_history_path() else {
        return Vec::new();
    };

    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };

    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            entries.push(trimmed.to_string());
        }
    }

    entries
}
