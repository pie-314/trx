use std::collections::{HashMap, HashSet};
use crate::managers::{Package, PackageManager};
use super::{pacman, yay};
use ratatui::DefaultTerminal;

pub struct ArchManager {
    pub aur_helper: String,
}

impl ArchManager {
    pub fn new(aur_helper: String) -> Self {
        Self { aur_helper }
    }
}

impl PackageManager for ArchManager {
    fn name(&self) -> &str {
        "Arch Linux (pacman/yay)"
    }

    fn search(&self, query: &str) -> Vec<Package> {
        // Check cache
        {
            let cache = crate::managers::SEARCH_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(query) {
                return cached.clone();
            }
        }

        let config = crate::config::Config::load();
        let enabled = &config.settings.enabled_managers;
        let show_pacman = enabled.contains(&"pacman".to_string());
        let show_yay = enabled.contains(&"yay".to_string());

        use rayon::prelude::*;

        let results: Vec<Vec<Package>> = vec![0, 1]
            .into_par_iter()
            .map(|i| {
                if i == 0 && show_pacman {
                    pacman::search_pacman(query)
                } else if i == 1 && show_yay {
                    yay::search_aur(query, &self.aur_helper)
                } else {
                    Vec::new()
                }
            })
            .collect();

        let mut all: Vec<Package> = results.into_iter().flatten().collect();
        all.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        all.truncate(config.settings.max_search_results);

        // Update cache
        {
            let mut cache = crate::managers::SEARCH_CACHE.lock().unwrap();
            cache.insert(query.to_string(), all.clone());
        }

        all
    }

    fn get_installed(&self) -> HashSet<String> {
        pacman::get_installed_packages()
    }

    fn get_installed_details(&self) -> Vec<Package> {
        pacman::get_installed_packages_details()
    }

    fn get_updates(&self) -> Vec<Package> {
        pacman::get_updates()
    }

    fn get_details(&self, pkg: &str, provider: &str) -> Option<HashMap<String, String>> {
        // Check cache first
        {
            let cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(pkg) {
                return Some(cached.clone());
            }
        }

        let pure_name = pkg.split('/').next_back().unwrap_or(pkg);
        let provide = provider.split('/').next().unwrap_or(provider);
        
        let info = match provide {
            "aur" => yay::aur_details(pure_name)?,
            "pacman" => pacman::pacman_info(pure_name)?,
            _ => return None,
        };

        // Update cache
        {
            let mut cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            cache.insert(pkg.to_string(), info.clone());
        }

        Some(info)
    }

    fn install(&self, terminal: &mut DefaultTerminal, pkgs: &HashSet<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut pacman_pkgs = HashSet::new();
        let mut aur_pkgs = HashSet::new();

        for name in pkgs {
            // Check provider prefix if possible, or just try to find in current list
            // For simplicity in the trait, we might need a better way to distinguish,
            // but for ArchManager we can check if it's in official repos first or has aur/ prefix
            if name.starts_with("aur/") {
                aur_pkgs.insert(name.clone());
            } else {
                pacman_pkgs.insert(name.clone());
            }
        }

        if !pacman_pkgs.is_empty() {
            pacman::pacman_install(terminal, &pacman_pkgs)?;
        }
        if !aur_pkgs.is_empty() {
            yay::aur_install(terminal, &aur_pkgs, &self.aur_helper)?;
        }
        Ok(())
    }

    fn remove(&self, terminal: &mut DefaultTerminal, pkgs: &HashSet<String>) -> Result<(), Box<dyn std::error::Error>> {
        pacman::pacman_remove(terminal, pkgs)
    }

    fn system_upgrade(&self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        let config = crate::config::Config::load();
        let enabled = &config.settings.enabled_managers;
        
        if enabled.contains(&"pacman".to_string()) {
            pacman::system_upgrade(terminal)?;
        }
        if enabled.contains(&"yay".to_string()) {
            yay::aur_upgrade(terminal, &self.aur_helper)?;
        }
        Ok(())
    }

    fn refresh_databases(&self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        pacman::refresh_databases(terminal)
    }
}
