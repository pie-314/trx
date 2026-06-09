use crate::managers::PackageManager;
use async_trait::async_trait;
use log::{debug, error, info, warn};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncBufReadExt;

/// Zypper package manager implementation for openSUSE
pub struct ZypperManager;

impl ZypperManager {
    /// Create a new ZypperManager instance
    pub fn new() -> Self {
        ZypperManager
    }

    /// Execute a zypper command and return the output
    async fn execute_zypper(args: &[&str]) -> Result<String, String> {
        debug!("Executing zypper with args: {:?}", args);
        
        let output = Command::new("zypper")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| format!("Failed to execute zypper: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Zypper exit codes:
        // 0 = success
        // 100 = updates available (not an error)
        // 1-6 = various error conditions
        if output.status.success() || output.status.code() == Some(100) {
            debug!("Zypper command succeeded (exit code: {:?})", output.status.code());
            Ok(stdout)
        } else {
            error!("Zypper command failed (exit code: {:?}): {}", output.status.code(), stderr);
            Err(format!("Zypper error: {}", stderr.trim()))
        }
    }

    /// Parse zypper --xmlout search output
    fn parse_search_xml(xml: &str) -> Vec<PackageInfo> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut packages = Vec::new();
        let mut current_package: Option<PackageInfo> = None;
        let mut current_field = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_field = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    if current_field == "solvable" {
                        current_package = Some(PackageInfo::default());
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Some(ref mut pkg) = current_package {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_field.as_str() {
                            "name" => pkg.name = text,
                            "version" => pkg.version = text,
                            "summary" => pkg.summary = text,
                            "description" => pkg.description = text,
                            "arch" => pkg.arch = text,
                            "repository" => pkg.repository = text,
                            "status" => pkg.status = text,
                            "kind" => pkg.kind = text,
                            _ => {}
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "solvable" {
                        if let Some(pkg) = current_package.take() {
                            packages.push(pkg);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("XML parsing error: {}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        packages
    }

    /// Parse zypper --xmlout list-updates output
    fn parse_updates_xml(xml: &str) -> Vec<PackageInfo> {
        Self::parse_search_xml(xml)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub arch: String,
    pub repository: String,
    pub status: String,
    pub kind: String,
}

#[async_trait]
impl PackageManager for ZypperManager {
    async fn search(&self, query: &str) -> Result<Vec<PackageInfo>, String> {
        info!("Searching for packages: {}", query);
        
        let output = self.execute_zypper(&["--xmlout", "search", query]).await?;
        let packages = Self::parse_search_xml(&output);
        
        debug!("Found {} packages matching '{}'", packages.len(), query);
        Ok(packages)
    }

    async fn install(&self, package: &str) -> Result<(), String> {
        info!("Installing package: {}", package);
        
        // Use --non-interactive for automated installation
        self.execute_zypper(&["install", "--non-interactive", "--no-confirm", package]).await?;
        
        info!("Successfully installed: {}", package);
        Ok(())
    }

    async fn remove(&self, package: &str) -> Result<(), String> {
        info!("Removing package: {}", package);
        
        self.execute_zypper(&["remove", "--non-interactive", "--no-confirm", package]).await?;
        
        info!("Successfully removed: {}", package);
        Ok(())
    }

    async fn update(&self, package: Option<&str>) -> Result<Vec<PackageInfo>, String> {
        match package {
            Some(pkg) => {
                info!("Updating package: {}", pkg);
                self.execute_zypper(&["update", "--non-interactive", "--no-confirm", pkg]).await?;
                info!("Successfully updated: {}", pkg);
                Ok(Vec::new())
            }
            None => {
                info!("Checking for system updates");
                
                // First check for available updates
                let output = self.execute_zypper(&["--xmlout", "list-updates"]).await?;
                let updates = Self::parse_updates_xml(&output);
                
                if !updates.is_empty() {
                    info!("Found {} available updates", updates.len());
                    
                    // Apply all updates
                    self.execute_zypper(&["update", "--non-interactive", "--no-confirm"]).await?;
                    info!("System updates applied successfully");
                } else {
                    info!("No updates available");
                }
                
                Ok(updates)
            }
        }
    }

    async fn list_installed(&self) -> Result<Vec<PackageInfo>, String> {
        info!("Listing installed packages");
        
        let output = self.execute_zypper(&["--xmlout", "search", "--installed-only"]).await?;
        let packages = Self::parse_search_xml(&output);
        
        debug!("Found {} installed packages", packages.len());
        Ok(packages)
    }

    async fn refresh(&self) -> Result<(), String> {
        info!("Refreshing package repositories");
        
        self.execute_zypper(&["refresh", "--force"]).await?;
        
        info!("Package repositories refreshed successfully");
        Ok(())
    }

    async fn info(&self, package: &str) -> Result<PackageInfo, String> {
        info!("Getting package info: {}", package);
        
        let output = self.execute_zypper(&["--xmlout", "info", package]).await?;
        let packages = Self::parse_search_xml(&output);
        
        packages.into_iter().next().ok_or_else(|| format!("Package '{}' not found", package))
    }

    async fn is_installed(&self, package: &str) -> Result<bool, String> {
        debug!("Checking if package is installed: {}", package);
        
        let output = self.execute_zypper(&["--xmlout", "search", "--installed-only", "--match-exact", package]).await?;
        let packages = Self::parse_search_xml(&output);
        
        Ok(!packages.is_empty())
    }

    async fn add_repository(&self, name: &str, url: &str) -> Result<(), String> {
        info!("Adding repository: {} ({})", name, url);
        
        self.execute_zypper(&["addrepo", "--no-gpgcheck", url, name]).await?;
        
        info!("Repository '{}' added successfully", name);
        Ok(())
    }

    async fn remove_repository(&self, name: &str) -> Result<(), String> {
        info!("Removing repository: {}", name);
        
        self.execute_zypper(&["removerepo", name]).await?;
        
        info!("Repository '{}' removed successfully", name);
        Ok(())
    }

    async fn list_repositories(&self) -> Result<Vec<String>, String> {
        info!("Listing repositories");
        
        let output = self.execute_zypper(&["--xmlout", "repos"]).await?;
        
        // Parse repository names from XML output
        let mut reader = Reader::from_str(&output);
        reader.trim_text(true);
        
        let mut repos = Vec::new();
        let mut in_repo = false;
        let mut current_field = String::new();
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    current_field = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if current_field == "repo" {
                        in_repo = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_repo && current_field == "name" {
                        let text = e.unescape().unwrap_or_default().to_string();
                        repos.push(text);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "repo" {
                        in_repo = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    warn!("XML parsing error: {}", e);
                    break;
                }
                _ => {}
            }
            buf.clear();
        }
        
        Ok(repos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_search_xml() {
        let xml = r#"<?xml version="1.0"?>
<stream>
<search-result version="0.0">
<solvable>
<name>vim</name>
<version>8.2.0-1.1</version>
<summary>Vi IMproved</summary>
<description>Vim is an advanced text editor</description>
<arch>x86_64</arch>
<repository>main-repo</repository>
<status>installed</status>
<kind>package</kind>
</solvable>
</search-result>
</stream>"#;

        let packages = ZypperManager::parse_search_xml(xml);
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "vim");
        assert_eq!(packages[0].version, "8.2.0-1.1");
        assert_eq!(packages[0].arch, "x86_64");
        assert_eq!(packages[0].status, "installed");
    }

    #[tokio::test]
    async fn test_parse_empty_search_xml() {
        let xml = r#"<?xml version="1.0"?>
<stream>
<search-result version="0.0">
</search-result>
</stream>"#;

        let packages = ZypperManager::parse_search_xml(xml);
        assert!(packages.is_empty());
    }

    #[tokio::test]
    async fn test_parse_multiple_packages() {
        let xml = r#"<?xml version="1.0"?>
<stream>
<search-result version="0.0">
<solvable>
<name>python3</name>
<version>3.9.0-1.1</version>
<summary>Python 3 interpreter</summary>
<arch>x86_64</arch>
<repository>main-repo</repository>
<status>not-installed</status>
<kind>package</kind>
</solvable>
<solvable>
<name>python3-pip</name>
<version>20.3.4-1.1</version>
<summary>Python package manager</summary>
<arch>noarch</arch>
<repository>main-repo</repository>
<status>not-installed</status>
<kind>package</kind>
</solvable>
</search-result>
</stream>"#;

        let packages = ZypperManager::parse_search_xml(xml);
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "python3");
        assert_eq!(packages[1].name, "python3-pip");
    }
}