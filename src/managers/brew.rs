use crate::managers::{Package, PackageManager, SEARCH_CACHE};
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

        let config = crate::config::Config::load();
        let max = config.settings.max_search_results;

        // Cache key includes the provider so CombinedManager searches don't
        // collide across backends that run the same user query string.
        let cache_key = format!("brew:{}", query);

        // Check cache first
        {
            let cache = SEARCH_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        // Step 1: get matching package names via `brew search`.
        // Overfetch slightly so that after score filtering we can still return
        // up to max_search_results entries.
        let fetch_limit = max.max(50);
        let names: Vec<String> = {
            let output = Command::new("brew").args(["search", "--formula", query]).output().ok();
            match output {
                Some(o) => String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && !l.starts_with("==>"))
                    .take(fetch_limit)
                    .collect(),
                None => return Vec::new(),
            }
        };

        if names.is_empty() {
            return Vec::new();
        }

        // Step 2: batch-fetch metadata via `brew info --json=v2`
        let info_map = fetch_brew_info_batch(&names);

        let mut results: Vec<Package> = names
            .iter()
            .map(|name| {
                let score = crate::fuzzy::fuzzy_match(query, name);
                if let Some((version, description)) = info_map.get(name.as_str()) {
                    Package {
                        provider: "brew".to_string(),
                        name: name.clone(),
                        version: version.clone(),
                        description: description.clone(),
                        score,
                    }
                } else {
                    Package {
                        provider: "brew".to_string(),
                        name: name.clone(),
                        version: String::new(),
                        description: String::new(),
                        score,
                    }
                }
            })
            .filter(|p| p.score > 0.01)
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max);

        // Update cache (keyed by provider+query so multi-manager setups don't collide)
        {
            let mut cache = SEARCH_CACHE.lock().unwrap();
            cache.insert(cache_key, results.clone());
        }

        results
    }

    fn get_installed(&self) -> HashSet<String> {
        let output = Command::new("brew").args(["list", "--formula", "--versions"]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
                .collect()
        } else {
            HashSet::new()
        }
    }

    fn get_installed_details(&self) -> Vec<Package> {
        let output = Command::new("brew").args(["list", "--formula", "--versions"]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    let mut parts = line.split_whitespace();
                    let name = parts.next()?.to_string();
                    // `brew list --formula --versions` may output multiple version
                    // tokens (e.g. "pkg 1.0 2.0"); take the last token as the
                    // most recently installed version.
                    let versions: Vec<&str> = parts.collect();
                    let version = versions.last().copied().unwrap_or("").to_string();
                    Some(Package {
                        provider: "brew".to_string(),
                        name,
                        version,
                        description: String::new(),
                        score: 1.0,
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn get_updates(&self) -> Vec<Package> {
        let output = Command::new("brew").args(["outdated", "--verbose"]).output().ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .filter_map(|line| {
                    // format: "name (installed_version) < latest_version"
                    let mut parts = line.splitn(2, ' ');
                    let name = parts.next()?.trim().to_string();
                    let version_info = parts.next().unwrap_or("").trim().to_string();
                    Some(Package {
                        provider: "brew".to_string(),
                        name,
                        version: version_info,
                        description: String::new(),
                        score: 1.0,
                    })
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

        let output = Command::new("brew").args(["info", "--json=v2", pkg]).output().ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;

        let formula = json["formulae"].as_array()?.first()?;
        let mut out = HashMap::new();

        if let Some(name) = formula["name"].as_str() {
            out.insert("Name".into(), name.into());
        }
        if let Some(ver) = formula["versions"]["stable"].as_str() {
            out.insert("Version".into(), ver.into());
        }
        if let Some(desc) = formula["desc"].as_str() {
            out.insert("Description".into(), desc.into());
        }
        if let Some(homepage) = formula["homepage"].as_str() {
            out.insert("Homepage".into(), homepage.into());
        }
        if let Some(license) = formula["license"].as_str() {
            out.insert("License".into(), license.into());
        }
        if let Some(tap) = formula["tap"].as_str() {
            out.insert("Tap".into(), tap.into());
        }
        if let Some(deps) = formula["dependencies"].as_array() {
            let dep_list: Vec<&str> = deps.iter().filter_map(|d| d.as_str()).collect();
            if !dep_list.is_empty() {
                out.insert("Dependencies".into(), dep_list.join(", "));
            }
        }
        if let Some(installed) = formula["installed"].as_array()
            && let Some(first) = installed.first()
            && let Some(installed_ver) = first["version"].as_str()
        {
            out.insert("Installed Version".into(), installed_ver.into());
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
        crate::execute_package_command(terminal, "brew", &args)?;
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
        crate::execute_package_command(terminal, "brew", &args)?;
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
        let mut args = vec!["upgrade"];
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

/// Batch-fetch version and description for a list of formula names using
/// `brew info --json=v2`. Falls back to an empty map on error.
fn fetch_brew_info_batch(names: &[String]) -> HashMap<String, (String, String)> {
    if names.is_empty() {
        return HashMap::new();
    }

    let mut args = vec!["info", "--json=v2"];
    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    args.extend(name_refs);

    let output = match Command::new("brew").args(&args).output() {
        Ok(o) => o,
        Err(_) => return HashMap::new(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(j) => j,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();
    if let Some(formulae) = json["formulae"].as_array() {
        for formula in formulae {
            let name = formula["name"].as_str().unwrap_or("").to_string();
            let version = formula["versions"]["stable"].as_str().unwrap_or("").to_string();
            let description = formula["desc"].as_str().unwrap_or("").to_string();
            if !name.is_empty() {
                map.insert(name, (version, description));
            }
        }
    }
    map
}
