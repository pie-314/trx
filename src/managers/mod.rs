pub mod apt;
pub mod dnf;
pub mod pacman;
pub mod zypper;

use async_trait::async_trait;

/// Represents information about a package
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

/// Trait that all package managers must implement
#[async_trait]
pub trait PackageManager: Send + Sync {
    /// Search for packages matching the query
    async fn search(&self, query: &str) -> Result<Vec<PackageInfo>, String>;
    
    /// Install a package
    async fn install(&self, package: &str) -> Result<(), String>;
    
    /// Remove a package
    async fn remove(&self, package: &str) -> Result<(), String>;
    
    /// Update packages. If package is None, update all packages
    async fn update(&self, package: Option<&str>) -> Result<Vec<PackageInfo>, String>;
    
    /// List all installed packages
    async fn list_installed(&self) -> Result<Vec<PackageInfo>, String>;
    
    /// Refresh package repositories
    async fn refresh(&self) -> Result<(), String>;
    
    /// Get detailed information about a package
    async fn info(&self, package: &str) -> Result<PackageInfo, String>;
    
    /// Check if a package is installed
    async fn is_installed(&self, package: &str) -> Result<bool, String>;
    
    /// Add a repository
    async fn add_repository(&self, name: &str, url: &str) -> Result<(), String>;
    
    /// Remove a repository
    async fn remove_repository(&self, name: &str) -> Result<(), String>;
    
    /// List all configured repositories
    async fn list_repositories(&self) -> Result<Vec<String>, String>;
}