use std::process::Command;

#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
}

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

//requires sudo
pub fn package_installer(package: &str) {
    println!("hello world");
    let cmd = format!("sudo pacman -S {}", package);
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute pacman -Qe");
}

// this gives details, return value is not yet defined, maybe hashmap
pub fn details_package(package: &str) {
    let cmd = format!("pacman -Si {}", package);
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute pacman -Qe");
}
