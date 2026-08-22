use std::process::Command;
use std::str;
use std::fmt;

use crate::managers::PackageManager;
use crate::models::Package;

/// Error type for DNF operations
#[derive(Debug)]
pub enum DnfError {
    CommandFailed(String),
    ParseError(String),
    PermissionDenied,
}

impl fmt::Display for DnfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnfError::CommandFailed(msg) => write!(f, "DNF command failed: {}", msg),
            DnfError::ParseError(msg) => write!(f, "Failed to parse DNF output: {}", msg),
            DnfError::PermissionDenied => write!(f, "Permission denied. Try running with sudo."),
        }
    }
}

impl std::error::Error for DnfError {}

/// DNF package manager implementation
pub struct Dnf;

impl Dnf {
    /// Create a new DNF instance
    pub fn new() -> Self {
        Dnf
    }

    /// Execute a DNF command and return the output
    fn execute_command(args: &[&str]) -> Result<String, DnfError> {
        let output = Command::new("dnf")
            .args(args)
            .output()
            .map_err(|e| DnfError::CommandFailed(format!("Failed to execute dnf: {}", e)))?;

        if !output.status.success() {
            let stderr = str::from_utf8(&output.stderr)
                .unwrap_or("Unknown error")
                .to_string();
            
            // Check for permission errors
            if stderr.contains("Permission denied") || stderr.contains("not in the sudoers file") {
                return Err(DnfError::PermissionDenied);
            }
            
            return Err(DnfError::CommandFailed(stderr));
        }

        str::from_utf8(&output.stdout)
            .map(|s| s.to_string())
            .map_err(|e| DnfError::ParseError(format!("Invalid UTF-8 output: {}", e)))
    }

    /// Execute a DNF command with sudo
    fn execute_sudo_command(args: &[&str]) -> Result<String, DnfError> {
        let output = Command::new("sudo")
            .arg("dnf")
            .args(args)
            .output()
            .map_err(|e| DnfError::CommandFailed(format!("Failed to execute sudo dnf: {}", e)))?;

        if !output.status.success() {
            let stderr = str::from_utf8(&output.stderr)
                .unwrap_or("Unknown error")
                .to_string();
            return Err(DnfError::CommandFailed(stderr));
        }

        str::from_utf8(&output.stdout)
            .map(|s| s.to_string())
            .map_err(|e| DnfError::ParseError(format!("Invalid UTF-8 output: {}", e)))
    }

    /// Parse a single line from `dnf list installed` or `dnf search` output
    fn parse_package_line(line: &str) -> Option<Package> {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Last metadata expiration") || 
           line.starts_with("Installed Packages") || line.starts_with("Available Packages") ||
           line.starts_with("====") || line.starts_with("---") {
            return None;
        }

        // DNF output format: "name.arch version repo"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let name_with_arch = parts[0];
        let version = parts[1].to_string();
        let repo = parts.get(2).map(|s| s.to_string()).unwrap_or_default();

        // Extract name and architecture from "name.arch"
        let (name, arch) = if let Some(dot_pos) = name_with_arch.rfind('.') {
            let name = name_with_arch[..dot_pos].to_string();
            let arch = name_with_arch[dot_pos + 1..].to_string();
            (name, arch)
        } else {
            (name_with_arch.to_string(), String::new())
        };

        Some(Package {
            name,
            version,
            arch,
            repo,
            description: String::new(),
            installed: false,
        })
    }

    /// Parse `dnf info` output to get detailed package information
    fn parse_info_output(output: &str) -> Vec<Package> {
        let mut packages = Vec::new();
        let mut current_pkg: Option<Package> = None;

        for line in output.lines() {
            let line = line.trim();
            
            if line.starts_with("Name ") {
                // Save previous package if exists
                if let Some(pkg) = current_pkg.take() {
                    packages.push(pkg);
                }
                let name = line.split(':').nth(1).unwrap_or("").trim().to_string();
                current_pkg = Some(Package {
                    name,
                    version: String::new(),
                    arch: String::new(),
                    repo: String::new(),
                    description: String::new(),
                    installed: false,
                });
            } else if let Some(ref mut pkg) = current_pkg {
                if line.starts_with("Version ") {
                    pkg.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Arch ") {
                    pkg.arch = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Repository ") {
                    pkg.repo = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Description ") {
                    pkg.description = line.split(':').nth(1).unwrap_or("").trim().to_string();
                } else if line.starts_with("Installed ") {
                    let val = line.split(':').nth(1).unwrap_or("").trim();
                    pkg.installed = val.eq_ignore_ascii_case("yes");
                }
            }
        }

        // Don't forget the last package
        if let Some(pkg) = current_pkg {
            packages.push(pkg);
        }

        packages
    }
}

impl PackageManager for Dnf {
    type Error = DnfError;

    /// Search for packages matching the query
    fn search(&self, query: &str) -> Result<Vec<Package>, Self::Error> {
        let output = Self::execute_command(&["search", query])?;
        
        let packages: Vec<Package> = output
            .lines()
            .filter_map(|line| Self::parse_package_line(line))
            .collect();

        Ok(packages)
    }

    /// Get list of installed packages
    fn get_installed(&self) -> Result<Vec<Package>, Self::Error> {
        let output = Self::execute_command(&["list", "installed"])?;
        
        let packages: Vec<Package> = output
            .lines()
            .filter_map(|line| Self::parse_package_line(line))
            .collect();

        Ok(packages)
    }

    /// Get detailed information about installed packages
    fn get_installed_details(&self) -> Result<Vec<Package>, Self::Error> {
        // First get the list of installed packages
        let installed = self.get_installed()?;
        
        if installed.is_empty() {
            return Ok(Vec::new());
        }

        // Get detailed info for each package using dnf info
        let package_names: Vec<&str> = installed.iter().map(|p| p.name.as_str()).collect();
        let mut args = vec!["info"];
        args.extend(package_names);
        
        let output = Self::execute_command(&args)?;
        let mut detailed_packages = Self::parse_info_output(&output);

        // Mark all as installed
        for pkg in &mut detailed_packages {
            pkg.installed = true;
        }

        Ok(detailed_packages)
    }

    /// Get list of available updates
    fn get_updates(&self) -> Result<Vec<Package>, Self::Error> {
        let output = Self::execute_command(&["check-update"])?;
        
        let packages: Vec<Package> = output
            .lines()
            .filter_map(|line| Self::parse_package_line(line))
            .collect();

        Ok(packages)
    }

    /// Install a package
    fn install(&self, package_name: &str) -> Result<(), Self::Error> {
        Self::execute_sudo_command(&["install", "-y", package_name])?;
        Ok(())
    }

    /// Remove a package
    fn remove(&self, package_name: &str) -> Result<(), Self::Error> {
        Self::execute_sudo_command(&["remove", "-y", package_name])?;
        Ok(())
    }

    /// Update all packages
    fn update_packages(&self) -> Result<(), Self::Error> {
        Self::execute_sudo_command(&["upgrade", "-y"])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_line_valid() {
        let line = "bash.x86_64 5.1.8-4.fc35 updates";
        let pkg = Dnf::parse_package_line(line).unwrap();
        assert_eq!(pkg.name, "bash");
        assert_eq!(pkg.version, "5.1.8-4.fc35");
        assert_eq!(pkg.arch, "x86_64");
        assert_eq!(pkg.repo, "updates");
    }

    #[test]
    fn test_parse_package_line_no_arch() {
        let line = "kernel 5.15.5-200.fc35 updates";
        let pkg = Dnf::parse_package_line(line).unwrap();
        assert_eq!(pkg.name, "kernel");
        assert_eq!(pkg.version, "5.15.5-200.fc35");
        assert_eq!(pkg.arch, "");
        assert_eq!(pkg.repo, "updates");
    }

    #[test]
    fn test_parse_package_line_empty() {
        assert!(Dnf::parse_package_line("").is_none());
        assert!(Dnf::parse_package_line("   ").is_none());
    }

    #[test]
    fn test_parse_package_line_header() {
        assert!(Dnf::parse_package_line("Installed Packages").is_none());
        assert!(Dnf::parse_package_line("Available Packages").is_none());
        assert!(Dnf::parse_package_line("Last metadata expiration check: 0:00:15 ago on Mon 22 Nov 2021 10:00:00 AM EST").is_none());
    }

    #[test]
    fn test_parse_info_output() {
        let output = r#"Name         : bash
Version      : 5.1.8-4.fc35
Arch         : x86_64
Repository   : @System
Description  : The GNU Bourne Again shell
Installed    : yes

Name         : vim
Version      : 8.2.3456-1.fc35
Arch         : x86_64
Repository   : updates
Description  : The VIM editor
Installed    : no
"#;

        let packages = Dnf::parse_info_output(output);
        assert_eq!(packages.len(), 2);
        
        assert_eq!(packages[0].name, "bash");
        assert_eq!(packages[0].version, "5.1.8-4.fc35");
        assert_eq!(packages[0].arch, "x86_64");
        assert_eq!(packages[0].repo, "@System");
        assert_eq!(packages[0].description, "The GNU Bourne Again shell");
        assert!(packages[0].installed);

        assert_eq!(packages[1].name, "vim");
        assert_eq!(packages[1].version, "8.2.3456-1.fc35");
        assert_eq!(packages[1].arch, "x86_64");
        assert_eq!(packages[1].repo, "updates");
        assert_eq!(packages[1].description, "The VIM editor");
        assert!(!packages[1].installed);
    }

    #[test]
    fn test_parse_info_output_empty() {
        let packages = Dnf::parse_info_output("");
        assert!(packages.is_empty());
    }

    #[test]
    fn test_error_display() {
        let err = DnfError::CommandFailed("test error".to_string());
        assert_eq!(format!("{}", err), "DNF command failed: test error");
        
        let err = DnfError::ParseError("parse error".to_string());
        assert_eq!(format!("{}", err), "Failed to parse DNF output: parse error");
        
        let err = DnfError::PermissionDenied;
        assert_eq!(format!("{}", err), "Permission denied. Try running with sudo.");
    }
}