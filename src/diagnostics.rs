use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DiagnosticEntry {
    pub tool: String,
    pub found: bool,
    pub version: String,
}

pub fn run_diagnostics() -> Vec<DiagnosticEntry> {
    let tools = ["apt", "brew", "pacman", "yay", "curl", "git"];
    let mut results = Vec::new();

    for tool in tools {
        let found = command_exists(tool);

        let version = if found { get_tool_version(tool) } else { "Not Found".to_string() };

        results.push(DiagnosticEntry { tool: tool.to_string(), found, version });
    }

    results.push(DiagnosticEntry {
        tool: "TRX".to_string(),
        found: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "trx") {
        let config_path = proj_dirs.config_dir().join("config.toml");

        results.push(DiagnosticEntry {
            tool: "Config".to_string(),
            found: Path::new(&config_path).exists(),
            version: config_path.display().to_string(),
        });
    }

    results
}

fn command_exists(tool: &str) -> bool {
    let check_command = if cfg!(windows) { "where" } else { "which" };

    Command::new(check_command)
        .arg(tool)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_tool_version(tool: &str) -> String {
    Command::new(tool)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|stdout| stdout.lines().next().map(|line| line.trim().to_string()))
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| "Unknown".to_string())
}
