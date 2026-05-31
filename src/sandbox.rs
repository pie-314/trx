#[cfg(target_os = "linux")]
use std::process::Command;

const BWRAP_WARNING: &str =
    "Warning: sandbox=true is set, but bubblewrap (bwrap) is not available on PATH.";

pub fn startup_warning(sandbox_enabled: bool) -> Option<&'static str> {
    if sandbox_enabled && !bwrap_available() {
        Some(BWRAP_WARNING)
    } else {
        None
    }
}

pub fn command_for(cmd: &str, args: &[&str], sandbox_enabled: bool) -> (String, Vec<String>) {
    if sandbox_enabled && bwrap_available() {
        ("bwrap".to_string(), bubblewrap_args(cmd, args))
    } else {
        (cmd.to_string(), args.iter().map(|arg| (*arg).to_string()).collect())
    }
}

#[cfg(target_os = "linux")]
pub fn bwrap_available() -> bool {
    Command::new("bwrap")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub fn bwrap_available() -> bool {
    false
}

fn bubblewrap_args(cmd: &str, args: &[&str]) -> Vec<String> {
    let mut wrapped = vec![
        "--ro-bind".to_string(),
        "/usr".to_string(),
        "/usr".to_string(),
        "--ro-bind".to_string(),
        "/etc".to_string(),
        "/etc".to_string(),
        "--bind".to_string(),
        "/tmp".to_string(),
        "/tmp".to_string(),
        "--proc".to_string(),
        "/proc".to_string(),
        "--dev".to_string(),
        "/dev".to_string(),
        "--".to_string(),
        cmd.to_string(),
    ];
    wrapped.extend(args.iter().map(|arg| (*arg).to_string()));
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_command_unchanged_when_sandbox_disabled() {
        let (cmd, args) = command_for("sudo", &["apt", "install", "ripgrep"], false);

        assert_eq!(cmd, "sudo");
        assert_eq!(args, vec!["apt", "install", "ripgrep"]);
    }

    #[test]
    fn warns_when_enabled_without_bwrap() {
        if !bwrap_available() {
            assert_eq!(startup_warning(true), Some(BWRAP_WARNING));
        }
    }

    #[test]
    fn builds_bubblewrap_args_with_original_command_after_separator() {
        let args = bubblewrap_args("apt", &["install", "ripgrep"]);

        assert!(args.starts_with(&[
            "--ro-bind".to_string(),
            "/usr".to_string(),
            "/usr".to_string(),
        ]));
        let separator = args.iter().position(|arg| arg == "--").unwrap();
        assert_eq!(
            &args[separator + 1..],
            vec![
                "apt".to_string(),
                "install".to_string(),
                "ripgrep".to_string()
            ]
        );
    }
}
