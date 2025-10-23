use std::process::Command;

/// A simple in-memory representation of a package returned by pacman queries.
/// Using a struct lets the UI work with structured data instead of raw printed text.
#[derive(Debug, Clone)]
pub struct Package {
    /// package name (e.g. `bash`)
    pub name: String,
    /// version string (e.g. `5.1.16-1`)
    pub version: String,
    /// short description line (from pacman output)
    pub description: String,
}

/// Parse lines where pacman output alternates between a header line and a description line,
/// matching the parsing approach used previously in this repo.
/// This keeps behavior similar to the original code but returns structured data.
fn parse_alternating_lines(lines: &[&str]) -> Vec<Package> {
    let mut res = Vec::new();
    let mut i = 0;
    while i + 1 < lines.len() {
        let first_line = lines[i];
        let second_line = lines[i + 1];

        // split first_line into tokens and expect at least "name version"
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let package = parts[0].to_string();
            let version = parts[1].to_string();
            let description = second_line.trim().to_string();
            res.push(Package {
                name: package,
                version,
                description,
            });
        }

        i += 2;
    }
    res
}

pub fn search_vec(search_word: &str) -> Vec<Package> {
    let cmd = format!("pacman -Ss {}", search_word);
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute pacman -Ss");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    parse_alternating_lines(&lines)
}

pub fn list_installed_vec() -> Vec<Package> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("pacman -Qe")
        .output()
        .expect("Failed to execute pacman -Qe");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    parse_alternating_lines(&lines)
}
