use crate::managers::{Package, PackageManager, SEARCH_CACHE};
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::process::Command;

pub struct FlatpakManager;

impl PackageManager for FlatpakManager {
    fn name(&self) -> &str {
        "Flatpak"
    }

    fn search(&self, query: &str) -> Vec<Package> {
        if query.is_empty() {
            return Vec::new();
        }

        let config = crate::config::Config::load();
        let max = config.settings.max_search_results;
        let cache_key = format!("flatpak:{}", query);

        {
            let cache = SEARCH_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        let output = Command::new("flatpak").args(["search", query]).output().ok();

        let Some(output) = output else {
            return Vec::new();
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();

        for line in stdout.lines().skip(1) {
            let columns: Vec<&str> = line.split('\t').collect();

            if columns.len() < 3 {
                continue;
            }

            let name = columns.first().unwrap_or(&"").trim();
            let description = columns.get(1).unwrap_or(&"").trim();
            let app_id = columns.get(2).unwrap_or(&"").trim();
            let version = columns.get(3).unwrap_or(&"").trim();

            if app_id.is_empty() {
                continue;
            }

            let score = crate::fuzzy::fuzzy_match(query, name)
                .max(crate::fuzzy::fuzzy_match(query, app_id));

            if score <= 0.01 {
                continue;
            }

            results.push(Package {
                provider: "flatpak".to_string(),
                name: app_id.to_string(),
                version: version.to_string(),
                description: description.to_string(),
                score,
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max);

        {
            let mut cache = SEARCH_CACHE.lock().unwrap();
            cache.insert(cache_key, results.clone());
        }

        results
    }

    fn get_installed(&self) -> HashSet<String> {
        let output =
            Command::new("flatpak").args(["list", "--app", "--columns=application"]).output().ok();

        let Some(output) = output else {
            return HashSet::new();
        };

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn get_installed_details(&self) -> Vec<Package> {
        let output = Command::new("flatpak")
            .args(["list", "--app", "--columns=application,version"])
            .output()
            .ok();

        let Some(output) = output else {
            return Vec::new();
        };

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let columns: Vec<&str> = line.split('\t').collect();
                let app_id = columns.first()?.trim();

                if app_id.is_empty() {
                    return None;
                }

                let version = columns.get(1).unwrap_or(&"").trim();

                Some(Package {
                    provider: "flatpak".to_string(),
                    name: app_id.to_string(),
                    version: version.to_string(),
                    description: String::new(),
                    score: 1.0,
                })
            })
            .collect()
    }

    fn get_updates(&self) -> Vec<Package> {
        let output = Command::new("flatpak")
            .args(["remote-ls", "--updates", "--columns=application,version"])
            .output()
            .ok();

        let Some(output) = output else {
            return Vec::new();
        };

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| {
                let columns: Vec<&str> = line.split('\t').collect();
                let app_id = columns.first()?.trim();

                if app_id.is_empty() {
                    return None;
                }

                let version = columns.get(1).unwrap_or(&"").trim();

                Some(Package {
                    provider: "flatpak".to_string(),
                    name: app_id.to_string(),
                    version: version.to_string(),
                    description: String::new(),
                    score: 1.0,
                })
            })
            .collect()
    }

    fn get_details(&self, pkg: &str, _provider: &str) -> Option<HashMap<String, String>> {
        {
            let cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(pkg) {
                return Some(cached.clone());
            }
        }

        let output = Command::new("flatpak").args(["info", pkg]).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut details = HashMap::new();

        details.insert("Application ID".to_string(), pkg.to_string());

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                if !key.is_empty() && !value.is_empty() {
                    details.insert(key.to_string(), value.to_string());
                }
            }
        }

        {
            let mut cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            cache.insert(pkg.to_string(), details.clone());
        }

        Some(details)
    }

    fn install(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for pkg in pkgs {
            crate::execute_external_command(terminal, "flatpak", &["install", "-y", pkg])?;
        }

        Ok(())
    }

    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for pkg in pkgs {
            crate::execute_external_command(terminal, "flatpak", &["uninstall", "-y", pkg])?;
        }

        Ok(())
    }

    fn update_packages(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for pkg in pkgs {
            crate::execute_external_command(terminal, "flatpak", &["update", "-y", pkg])?;
        }

        Ok(())
    }

    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "flatpak", &["update", "-y"])?;
        Ok(())
    }

    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "flatpak", &["update", "--appstream"])?;
        Ok(())
    }
}
