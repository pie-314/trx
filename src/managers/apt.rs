use crate::managers::{Package, PackageManager};
use ratatui::DefaultTerminal;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::process::Command;

pub struct AptManager;

fn parse_search_line(query: &str, line: &str) -> Option<Package> {
    let parts: Vec<&str> = line.splitn(2, " - ").collect();
    if parts.is_empty() {
        return None;
    }

    let name = parts[0].trim().to_string();
    if name.is_empty() {
        return None;
    }

    let description = parts.get(1).unwrap_or(&"").trim().to_string();
    let score = crate::fuzzy::fuzzy_match(query, &name);

    if score <= 0.01 {
        return None;
    }

    Some(Package { provider: "apt".to_string(), name, version: String::new(), description, score })
}

fn parse_installed_detail_line(line: &str) -> Option<Package> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 2 {
        return None;
    }

    Some(Package {
        provider: "apt/local".to_string(),
        name: parts[0].to_string(),
        version: parts[1].to_string(),
        description: parts.get(2).unwrap_or(&"").to_string(),
        score: 1.0,
    })
}

fn parse_update_line(line: &str) -> Option<Package> {
    let parts: Vec<&str> = line.split('/').collect();
    if parts.is_empty() || parts[0].is_empty() {
        return None;
    }

    Some(Package {
        provider: "apt/update".to_string(),
        name: parts[0].to_string(),
        version: "Update available".to_string(),
        description: String::new(),
        score: 1.0,
    })
}

impl PackageManager for AptManager {
    fn name(&self) -> &str {
        "APT (Debian/Ubuntu)"
    }

    fn search(&self, query: &str) -> Vec<Package> {
        if query.is_empty() {
            return Vec::new();
        }
        let output = Command::new("apt-cache").args(["search", query]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            lines.par_iter().filter_map(|line| parse_search_line(query, line)).collect()
        } else {
            Vec::new()
        }
    }

    fn get_installed(&self) -> HashSet<String> {
        let output = Command::new("dpkg-query").args(["-W", "-f=${Package}\n"]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().map(|s| s.trim().to_string()).collect()
        } else {
            HashSet::new()
        }
    }

    fn get_installed_details(&self) -> Vec<Package> {
        let output = Command::new("dpkg-query")
            .args(["-W", "-f=${Package}\t${Version}\t${Description}\n"])
            .output()
            .ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();
            lines.par_iter().filter_map(|line| parse_installed_detail_line(line)).collect()
        } else {
            Vec::new()
        }
    }

    fn get_updates(&self) -> Vec<Package> {
        let output = Command::new("apt").args(["list", "--upgradable"]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().skip(1).collect();
            lines.par_iter().filter_map(|line| parse_update_line(line)).collect()
        } else {
            Vec::new()
        }
    }

    fn get_details(&self, pkg: &str, _provider: &str) -> Option<HashMap<String, String>> {
        // Check cache first
        {
            let cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(pkg) {
                return Some(cached.clone());
            }
        }

        let output = Command::new("apt-cache").args(["show", pkg]).output().ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut out = HashMap::new();
        out.insert("Info".to_string(), stdout.to_string());

        // Update cache
        {
            let mut cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            cache.insert(pkg.to_string(), out.clone());
        }

        Some(out)
    }

    fn install(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut args = vec!["apt", "install", "-y"];
        let pkg_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_package_command(terminal, "sudo", &args)?;
        Ok(())
    }

    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut args = vec!["apt", "remove", "-y"];
        let pkg_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_package_command(terminal, "sudo", &args)?;
        Ok(())
    }

    fn update_packages(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if pkgs.is_empty() {
            return Ok(());
        }
        let mut args = vec!["apt", "install", "--only-upgrade", "-y"];
        let pkg_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_external_command(terminal, "sudo", &args)?;
        Ok(())
    }

    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "sudo", &["apt", "upgrade", "-y"])?;
        Ok(())
    }

    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "sudo", &["apt", "update"])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apt_search_lines_including_empty_descriptions() {
        let with_description = parse_search_line("curl", "curl - command line tool").unwrap();
        assert_eq!(with_description.provider, "apt");
        assert_eq!(with_description.name, "curl");
        assert_eq!(with_description.description, "command line tool");

        let without_description = parse_search_line("curl", "curl").unwrap();
        assert_eq!(without_description.name, "curl");
        assert_eq!(without_description.description, "");
    }

    #[test]
    fn rejects_empty_apt_search_names() {
        assert!(parse_search_line("curl", " - missing package").is_none());
    }

    #[test]
    fn parses_apt_detail_and_update_lines() {
        let detail = parse_installed_detail_line("curl\t8.5.0\ttransfer tool").unwrap();
        assert_eq!(detail.provider, "apt/local");
        assert_eq!(detail.name, "curl");
        assert_eq!(detail.version, "8.5.0");
        assert_eq!(detail.description, "transfer tool");

        let update = parse_update_line("curl/stable 8.6.0 amd64 [upgradable]").unwrap();
        assert_eq!(update.provider, "apt/update");
        assert_eq!(update.name, "curl");
        assert_eq!(update.version, "Update available");
    }
}
