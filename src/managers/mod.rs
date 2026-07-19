pub mod apt;
pub mod arch;
pub mod brew;
pub mod pacman;
pub mod yay;
pub mod zypper;

use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub provider: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub score: f64,
}

pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn search(&self, query: &str) -> Vec<Package>;
    fn get_installed(&self) -> HashSet<String>;
    fn get_installed_details(&self) -> Vec<Package>;
    fn get_updates(&self) -> Vec<Package>;
    fn get_details(&self, pkg: &str, provider: &str) -> Option<HashMap<String, String>>;
    fn install(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn update_packages(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct CombinedManager {
    managers: Vec<Box<dyn PackageManager>>,
}

impl PackageManager for CombinedManager {
    fn name(&self) -> &str {
        "Multiple Managers"
    }

    fn search(&self, query: &str) -> Vec<Package> {
        let mut all = Vec::new();
        for m in &self.managers {
            all.extend(m.search(query));
        }
        all.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        all
    }

    fn get_installed(&self) -> HashSet<String> {
        let mut all = HashSet::new();
        for m in &self.managers {
            all.extend(m.get_installed());
        }
        all
    }

    fn get_installed_details(&self) -> Vec<Package> {
        let mut all = Vec::new();
        for m in &self.managers {
            all.extend(m.get_installed_details());
        }
        all
    }

    fn get_updates(&self) -> Vec<Package> {
        let mut all = Vec::new();
        for m in &self.managers {
            all.extend(m.get_updates());
        }
        all
    }

    fn get_details(&self, pkg: &str, provider: &str) -> Option<HashMap<String, String>> {
        for m in &self.managers {
            // Check if this manager's name/prefix matches the provider
            if manager_handles_provider(m.name(), provider) {
                return m.get_details(pkg, provider);
            }
        }
        // Fallback: try all
        for m in &self.managers {
            if let Some(details) = m.get_details(pkg, provider) {
                return Some(details);
            }
        }
        None
    }

    fn install(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for m in &self.managers {
            // This is tricky; we'd need to know which package belongs to which manager.
            // For now, let's assume the manager's internal logic handles its own packages.
            m.install(terminal, pkgs)?;
        }
        Ok(())
    }

    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for m in &self.managers {
            m.remove(terminal, pkgs)?;
        }
        Ok(())
    }

    fn update_packages(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Partition `pkgs` by manager: only send a package to the backend that
        // owns it (determined by intersecting with each manager's installed set).
        // This prevents backends from attempting to upgrade packages they don't manage.
        for m in &self.managers {
            let installed = m.get_installed();
            let manager_pkgs: HashSet<String> =
                pkgs.iter().filter(|p| installed.contains(*p)).cloned().collect();
            if !manager_pkgs.is_empty() {
                m.update_packages(terminal, &manager_pkgs)?;
            }
        }
        Ok(())
    }

    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for m in &self.managers {
            m.system_upgrade(terminal)?;
        }
        Ok(())
    }

    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for m in &self.managers {
            m.refresh_databases(terminal)?;
        }
        Ok(())
    }
}

pub fn get_available_managers() -> Vec<String> {
    let mut available = Vec::new();

    if std::process::Command::new("pacman").arg("--version").output().is_ok() {
        available.push("pacman".to_string());
        available.push("yay".to_string());
    }

    if std::process::Command::new("brew").arg("--version").output().is_ok() {
        available.push("brew".to_string());
    }

    if std::process::Command::new("apt").arg("--version").output().is_ok() {
        available.push("apt".to_string());
    }

    // Zypper does not have a reliable quick exit like `--version` that 
    // doesn't potentially trigger a slow refresh or output verbose messages.
    // So we check for its presence in PATH via `which zypper` as the safest and fastest methodology.
    if std::process::Command::new("which")
        .arg("zypper")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        available.push("zypper".to_string());
    }

    available
}

pub fn get_system_manager(config: &crate::config::Config) -> Box<dyn PackageManager> {
    let mut managers: Vec<Box<dyn PackageManager>> = Vec::new();
    let enabled = &config.settings.enabled_managers;
    let available = get_available_managers();

    if available.contains(&"brew".to_string()) && enabled.contains(&"brew".to_string()) {
        managers.push(Box::new(brew::BrewManager));
    }

    if (available.contains(&"pacman".to_string()) || available.contains(&"yay".to_string()))
        && (enabled.contains(&"pacman".to_string()) || enabled.contains(&"yay".to_string()))
    {
        managers.push(Box::new(arch::ArchManager::new(config.aur_helper.clone())));
    }

    if available.contains(&"apt".to_string()) && enabled.contains(&"apt".to_string()) {
        managers.push(Box::new(apt::AptManager));
    }

    if available.contains(&"zypper".to_string()) && enabled.contains(&"zypper".to_string()) {
        managers.push(Box::new(zypper::ZypperManager));
    }

    if managers.is_empty() {
        if available.contains(&"pacman".to_string()) {
            return Box::new(arch::ArchManager::new(config.aur_helper.clone()));
        }
        return Box::new(apt::AptManager);
    }

    if managers.len() == 1 {
        return managers.pop().unwrap();
    }

    Box::new(CombinedManager { managers })
}

//makes a DETAILS_CACHE which is global
lazy_static::lazy_static! {
    pub static ref DETAILS_CACHE: Arc<Mutex<HashMap<String, HashMap<String, String>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref SEARCH_CACHE: Arc<Mutex<HashMap<String, Vec<Package>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub fn parse_alternating_lines(lines: &[&str], manager: String, query: &str) -> Vec<Package> {
    let mut res = Vec::new();
    let mut i = 0;

    while i + 1 < lines.len() {
        let first_line = lines[i];
        let second_line = lines[i + 1];

        let parts: Vec<&str> = first_line.split_whitespace().collect();

        if parts.len() >= 2 {
            let package = parts[0].to_string();
            let version = parts[1].to_string();
            let description = second_line.trim().to_string();

            let package_name = package.split('/').next_back().unwrap_or(&package).to_string();
            let score = crate::fuzzy::fuzzy_match(query, &package_name);

            res.push(Package {
                provider: manager.clone(),
                name: package,
                version,
                description,
                score,
            });
        }

        i += 2;
    }

    res.retain(|p| p.score > 0.01);
    res.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    res
}

pub fn manager_handles_provider(manager_name: &str, provider: &str) -> bool {
    let m_lower = manager_name.to_lowercase();
    let p_lower = provider.to_lowercase();
    if p_lower.contains("pacman") || p_lower.contains("aur") || p_lower.contains("yay") {
        m_lower.contains("arch") || m_lower.contains("pacman") || m_lower.contains("yay")
    } else if p_lower.contains("brew") || p_lower.contains("homebrew") {
        m_lower.contains("brew")
    } else if p_lower.contains("apt") {
        m_lower.contains("apt") || m_lower.contains("debian") || m_lower.contains("ubuntu")
    } else if p_lower.contains("zypper") || p_lower.contains("opensuse") || p_lower.contains("suse")
    {
        m_lower.contains("zypper") || m_lower.contains("opensuse") || m_lower.contains("suse")
    } else {
        p_lower.contains(&m_lower) || m_lower.contains(&p_lower)
    }
}
