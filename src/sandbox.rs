use crate::config::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub cmd: String,
    pub args: Vec<String>,
}

const BWRAP_ARGS: &[&str] = &[
    "--ro-bind",
    "/usr",
    "/usr",
    "--ro-bind",
    "/etc",
    "/etc",
    "--bind",
    "/tmp",
    "/tmp",
    "--proc",
    "/proc",
    "--dev",
    "/dev",
    "--",
];

pub fn build_package_command(
    cmd: &str,
    args: &[&str],
    sandbox_enabled: bool,
    bwrap_available: bool,
    target_linux: bool,
) -> CommandSpec {
    if !(sandbox_enabled && bwrap_available && target_linux) {
        return CommandSpec {
            cmd: cmd.to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        };
    }

    let mut sandbox_args: Vec<String> = BWRAP_ARGS.iter().map(|arg| (*arg).to_string()).collect();
    sandbox_args.push(cmd.to_string());
    sandbox_args.extend(args.iter().map(|arg| (*arg).to_string()));

    CommandSpec { cmd: "bwrap".to_string(), args: sandbox_args }
}

pub fn package_command(cmd: &str, args: &[&str]) -> CommandSpec {
    let config = Config::load();
    build_package_command(
        cmd,
        args,
        config.settings.sandbox,
        is_bwrap_available(),
        cfg!(target_os = "linux"),
    )
}

pub fn warn_if_misconfigured(config: &Config) {
    if config.settings.sandbox && cfg!(target_os = "linux") && !is_bwrap_available() {
        eprintln!(
            "warning: sandbox=true but Bubblewrap (bwrap) was not found in PATH; package commands will run without sandboxing"
        );
    }
}

fn is_bwrap_available() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }

    std::process::Command::new("bwrap").arg("--version").output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_command_unchanged_when_sandbox_disabled() {
        let command =
            build_package_command("sudo", &["apt", "install", "ripgrep"], false, true, true);

        assert_eq!(command.cmd, "sudo");
        assert_eq!(command.args, vec!["apt", "install", "ripgrep"]);
    }

    #[test]
    fn leaves_command_unchanged_when_bwrap_is_missing() {
        let command =
            build_package_command("sudo", &["apt", "remove", "ripgrep"], true, false, true);

        assert_eq!(command.cmd, "sudo");
        assert_eq!(command.args, vec!["apt", "remove", "ripgrep"]);
    }

    #[test]
    fn leaves_command_unchanged_on_non_linux_targets() {
        let command = build_package_command("brew", &["install", "ripgrep"], true, true, false);

        assert_eq!(command.cmd, "brew");
        assert_eq!(command.args, vec!["install", "ripgrep"]);
    }

    #[test]
    fn wraps_linux_package_command_when_enabled_and_available() {
        let command =
            build_package_command("sudo", &["apt", "install", "ripgrep"], true, true, true);

        assert_eq!(command.cmd, "bwrap");
        assert_eq!(
            command.args,
            vec![
                "--ro-bind",
                "/usr",
                "/usr",
                "--ro-bind",
                "/etc",
                "/etc",
                "--bind",
                "/tmp",
                "/tmp",
                "--proc",
                "/proc",
                "--dev",
                "/dev",
                "--",
                "sudo",
                "apt",
                "install",
                "ripgrep",
            ]
        );
    }
}
