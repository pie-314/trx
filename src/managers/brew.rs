use crate::managers::{Package, PackageManager};
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::process::Command;

pub struct BrewManager;

impl PackageManager for BrewManager {
    fn name(&self) -> &str {
        "Homebrew (macOS/Linux)"
    }

    fn search(&self, query: &str) -> Vec<Package> {
        if query.is_empty() {
            return Vec::new();
        }
        let output = Command::new("brew").args(["search", query]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(|line| {
                    let name = line.trim().to_string();
                    let score = crate::fuzzy::fuzzy_match(query, &name);
                    Package {
                        provider: "brew".to_string(),
                        name,
                        version: "".to_string(), // search doesn't give version
                        description: "".to_string(), // search doesn't give desc
                        score,
                    }
                })
                .filter(|p| p.score > 0.01)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_installed(&self) -> HashSet<String> {
        let output = Command::new("brew")
            .args(["list", "--formula"])
            .output()
            .ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().map(|s| s.trim().to_string()).collect()
        } else {
            HashSet::new()
        }
    }

    fn get_installed_details(&self) -> Vec<Package> {
        let output = Command::new("brew")
            .args(["list", "--formula"])
            .output()
            .ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(|line| Package {
                    provider: "brew".to_string(),
                    name: line.trim().to_string(),
                    version: "Installed".to_string(),
                    description: "".to_string(),
                    score: 1.0,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_updates(&self) -> Vec<Package> {
        let output = Command::new("brew").args(["outdated"]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(|line| Package {
                    provider: "brew".to_string(),
                    name: line.trim().to_string(),
                    version: "Outdated".to_string(),
                    description: "".to_string(),
                    score: 1.0,
                })
                .collect()
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

        let output = Command::new("brew").args(["info", pkg]).output().ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut out = HashMap::new();
        
        let lines: Vec<&str> = stdout.lines().collect();
        if lines.is_empty() {
            return None;
        }

        // Line 1: name: version (status)
        if let Some(first_line) = lines.first() {
            let parts: Vec<&str> = first_line.splitn(2, ':').collect();
            if parts.len() == 2 {
                out.insert("Name".into(), parts[0].trim().into());
                out.insert("Version".into(), parts[1].trim().into());
            } else {
                out.insert("Info".into(), first_line.trim().into());
            }
        }

        // Line 2: Description
        if let Some(second_line) = lines.get(1) {
            out.insert("Description".into(), second_line.trim().into());
        }

        // Line 3: URL
        if let Some(third_line) = lines.get(2) {
            out.insert("URL".into(), third_line.trim().into());
        }

        // Subsequent lines: License, From, Caveats, etc.
        let mut caveats = Vec::new();
        let mut in_caveats = false;
        let mut analytics = Vec::new();
        let mut in_analytics = false;

        for line in lines.iter().skip(3) {
            let l = line.trim();
            if l.is_empty() { continue; }

            if l.starts_with("License:") {
                out.insert("License".into(), l.replace("License:", "").trim().into());
            } else if l.starts_with("From:") {
                out.insert("From".into(), l.replace("From:", "").trim().into());
            } else if l.starts_with("==> Caveats") {
                in_caveats = true;
                in_analytics = false;
            } else if l.starts_with("==> Analytics") {
                in_analytics = true;
                in_caveats = false;
            } else if l.starts_with("==>") {
                in_caveats = false;
                in_analytics = false;
            } else if in_caveats {
                caveats.push(l);
            } else if in_analytics {
                analytics.push(l);
            }
        }

        if !caveats.is_empty() {
            out.insert("Caveats".into(), caveats.join(" "));
        }
        if !analytics.is_empty() {
            out.insert("Analytics".into(), analytics.join(" | "));
        }

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
        let mut args = vec!["install"];
        let pkg_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_external_command(terminal, "brew", &args)?;
        Ok(())
    }

    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut args = vec!["uninstall"];
        let pkg_refs: Vec<&str> = pkgs.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_external_command(terminal, "brew", &args)?;
        Ok(())
    }

    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "brew", &["upgrade"])?;
        Ok(())
    }

    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "brew", &["update"])?;
        Ok(())
    }
}
