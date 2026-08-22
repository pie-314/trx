/// Trait defining the interface for package managers
pub trait PackageManager {
    type Error: std::error::Error;

    /// Search for packages matching the query
    fn search(&self, query: &str) -> Result<Vec<Package>, Self::Error>;

    /// Get list of installed packages
    fn get_installed(&self) -> Result<Vec<Package>, Self::Error>;

    /// Get detailed information about installed packages
    fn get_installed_details(&self) -> Result<Vec<Package>, Self::Error>;

    /// Get list of available updates
    fn get_updates(&self) -> Result<Vec<Package>, Self::Error>;

    /// Install a package
    fn install(&self, package_name: &str) -> Result<(), Self::Error>;

    /// Remove a package
    fn remove(&self, package_name: &str) -> Result<(), Self::Error>;

    /// Update all packages
    fn update_packages(&self) -> Result<(), Self::Error>;
}