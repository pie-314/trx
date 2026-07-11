use crate::managers::{Package, PackageManager};
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::process::Command;

pub struct ZypperManager;

/// Parse zypper XML output for `search` and `packages` commands.
/// Zypper XML search results have the form:
/// ```xml
/// <solvable name="vim" kind="package" edition="9.1" arch="x86_64" summary="Vi IMproved"/>
/// ```
fn parse_xml_solvables(xml: &str, provider: &str, query: &str) -> Vec<Package> {
    let mut packages = Vec::new();

    for line in xml.lines() {
        let line = line.trim();
        if !line.starts_with("<solvable ") {
            continue;
        }

        let name = extract_xml_attr(line, "name").unwrap_or_default();
        let edition = extract_xml_attr(line, "edition").unwrap_or_default();
        let summary = extract_xml_attr(line, "summary").unwrap_or_default();

        if name.is_empty() {
            continue;
        }

        let score = if query.is_empty() {
            1.0
        } else {
            crate::fuzzy::fuzzy_match(query, &name)
        };

        packages.push(Package {
            provider: provider.to_string(),
            name,
            version: edition,
            description: summary,
            score,
        });
    }

    packages
}

/// Minimal attribute extractor for zypper XML (avoids a full XML parser dependency).
/// Handles both single- and double-quoted attribute values.
fn extract_xml_attr<'a>(line: &'a str, attr: &str) -> Option<String> {
    let needle = format!("{}=\"", attr);
    if let Some(start) = line.find(&needle) {
        let rest = &line[start + needle.len()..];
        let end = rest.find('"')?;
        return Some(rest[..end].to_string());
    }
    // Try single-quoted variant
    let needle_sq = format!("{}='", attr);
    if let Some(start) = line.find(&needle_sq) {
        let rest = &line[start + needle_sq.len()..];
        let end = rest.find('\'')?;
        return Some(rest[..end].to_string());
    }
    None
}

impl PackageManager for ZypperManager {
    fn name(&self) -> &str {
        "Zypper (openSUSE)"
    }

    // -------------------------------------------------------------------------
    // search
    // -------------------------------------------------------------------------
    fn search(&self, query: &str) -> Vec<Package> {
        if query.is_empty() {
            return Vec::new();
        }

        // Check cache
        {
            let cache = crate::managers::SEARCH_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(query) {
                return cached.clone();
            }
        }

        let output = Command::new("zypper")
            .args(["--xmlout", "search", query])
            .output()
            .ok();

        let mut packages = if let Some(out) = output {
            // zypper exits with 0 (no results) or non-zero on real errors;
            // any XML in stdout is still valid regardless of exit code here.
            let xml = String::from_utf8_lossy(&out.stdout);
            let mut pkgs = parse_xml_solvables(&xml, "zypper", query);
            pkgs.retain(|p| p.score > 0.01);
            pkgs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
            pkgs
        } else {
            Vec::new()
        };

        let config = crate::config::Config::load();
        packages.truncate(config.settings.max_search_results);

        // Update cache
        {
            let mut cache = crate::managers::SEARCH_CACHE.lock().unwrap();
            cache.insert(query.to_string(), packages.clone());
        }

        packages
    }

    // -------------------------------------------------------------------------
    // get_installed  (names only, used for highlighting)
    // -------------------------------------------------------------------------
    fn get_installed(&self) -> HashSet<String> {
        let output = Command::new("zypper")
            .args(["--xmlout", "search", "-i"])
            .output()
            .ok();

        if let Some(out) = output {
            let xml = String::from_utf8_lossy(&out.stdout);
            parse_xml_solvables(&xml, "zypper", "")
                .into_iter()
                .map(|p| p.name)
                .collect()
        } else {
            HashSet::new()
        }
    }

    // -------------------------------------------------------------------------
    // get_installed_details
    // -------------------------------------------------------------------------
    fn get_installed_details(&self) -> Vec<Package> {
        let output = Command::new("zypper")
            .args(["--xmlout", "search", "-i"])
            .output()
            .ok();

        if let Some(out) = output {
            let xml = String::from_utf8_lossy(&out.stdout);
            let mut pkgs = parse_xml_solvables(&xml, "zypper/local", "");
            pkgs.iter_mut().for_each(|p| p.score = 1.0);
            pkgs
        } else {
            Vec::new()
        }
    }

    // -------------------------------------------------------------------------
    // get_updates
    // Exit code 100 means "updates available" — that is NOT an error for zypper.
    // -------------------------------------------------------------------------
    fn get_updates(&self) -> Vec<Package> {
        let output = Command::new("zypper")
            .args(["--xmlout", "list-updates"])
            .output()
            .ok();

        if let Some(out) = output {
            // Exit code 100 = updates available (treat like success)
            let code = out.status.code().unwrap_or(0);
            if code != 0 && code != 100 {
                return Vec::new();
            }
            let xml = String::from_utf8_lossy(&out.stdout);
            let mut pkgs = parse_xml_solvables(&xml, "zypper/update", "");
            pkgs.iter_mut().for_each(|p| {
                p.score = 1.0;
                if p.description.is_empty() {
                    p.description = "Update available".to_string();
                }
            });
            pkgs
        } else {
            Vec::new()
        }
    }

    // -------------------------------------------------------------------------
    // get_details  (plain-text `info` sub-command; XML info is verbose)
    // -------------------------------------------------------------------------
    fn get_details(&self, pkg: &str, _provider: &str) -> Option<HashMap<String, String>> {
        // Check cache first
        {
            let cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(pkg) {
                return Some(cached.clone());
            }
        }

        let pure_name = pkg.split('/').next_back().unwrap_or(pkg);

        let output = Command::new("zypper")
            .args(["info", pure_name])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let mut info = HashMap::new();
        info.insert("Info".to_string(), stdout);

        // Update cache
        {
            let mut cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            cache.insert(pkg.to_string(), info.clone());
        }

        Some(info)
    }

    // -------------------------------------------------------------------------
    // install
    // -------------------------------------------------------------------------
    fn install(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<(String, String)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let names: HashSet<String> = pkgs
            .iter()
            .filter(|(_, provider)| super::manager_handles_provider(self.name(), provider))
            .map(|(name, _)| name.clone())
            .collect();
        if names.is_empty() {
            return Ok(());
        }
        let mut args = vec!["zypper", "install", "-y"];
        let pkg_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_external_command(terminal, "sudo", &args)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // remove
    // -------------------------------------------------------------------------
    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<(String, String)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let names: HashSet<String> = pkgs
            .iter()
            .filter(|(_, provider)| super::manager_handles_provider(self.name(), provider))
            .map(|(name, _)| name.clone())
            .collect();
        if names.is_empty() {
            return Ok(());
        }
        let mut args = vec!["zypper", "remove", "-y"];
        let pkg_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_external_command(terminal, "sudo", &args)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // update_packages  (install a specific version == upgrade selected pkgs)
    // -------------------------------------------------------------------------
    fn update_packages(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<(String, String)>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let names: HashSet<String> = pkgs
            .iter()
            .filter(|(_, provider)| super::manager_handles_provider(self.name(), provider))
            .map(|(name, _)| name.clone())
            .collect();
        if names.is_empty() {
            return Ok(());
        }
        let mut args = vec!["zypper", "update", "-y"];
        let pkg_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        args.extend(pkg_refs);
        crate::execute_external_command(terminal, "sudo", &args)?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // system_upgrade  →  zypper dup  (distribution upgrade)
    // -------------------------------------------------------------------------
    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "sudo", &["zypper", "dup", "-y"])?;
        Ok(())
    }

    // -------------------------------------------------------------------------
    // refresh_databases  →  zypper refresh
    // -------------------------------------------------------------------------
    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "sudo", &["zypper", "refresh"])?;
        Ok(())
    }
}
