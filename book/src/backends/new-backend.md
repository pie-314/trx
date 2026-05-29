# Adding a New Backend

TRX is designed to make adding a new package manager straightforward. You only need to:

1. Create a new file in `src/managers/`
2. Implement the `PackageManager` trait
3. Register the backend in `get_system_manager`

---

## Step 1 — Create the backend file

```
src/managers/dnf.rs    # example: Fedora/RHEL dnf
```

Add the module to `src/managers/mod.rs`:

```rust
pub mod dnf;
```

---

## Step 2 — Implement `PackageManager`

### Understanding the PackageManager Trait

Every backend must implement all methods defined in the `PackageManager` trait.

### name()

Returns a human-readable name for the package manager.

Example:
- "APT (Debian/Ubuntu)"
- "Homebrew (macOS/Linux)"

### search(query)

Searches for packages matching the provided query and returns a `Vec<Package>`.

Expected behavior:
- Execute the package manager's search command
- Parse command output
- Calculate fuzzy-match scores
- Return matching packages

### get_installed()

Returns all installed package names as a `HashSet<String>`.

### get_installed_details()

Returns detailed information about installed packages, including version and description.

### get_updates()

Returns packages that have updates available.

### get_details(pkg, provider)

Returns detailed metadata for a package.

Expected behavior:
- Check `DETAILS_CACHE` before making external calls
- Store fetched results in the cache
- Return package metadata as a `HashMap<String, String>`

### install()

Installs the selected packages.

### remove()

Removes the selected packages.

### update_packages()

Updates only the selected packages.

### system_upgrade()

Performs a full system upgrade using the package manager.

### refresh_databases()

Refreshes package indexes and metadata before searches or upgrades.

Below is a minimal skeleton. All methods must be implemented (the trait has no default implementations):

```rust
use crate::managers::{Package, PackageManager};
use ratatui::DefaultTerminal;
use std::collections::{HashMap, HashSet};
use std::process::Command;

pub struct DnfManager;

impl PackageManager for DnfManager {
    fn name(&self) -> &str {
        "DNF (Fedora/RHEL)"
    }

    fn search(&self, query: &str) -> Vec<Package> {
        if query.is_empty() {
            return Vec::new();
        }
        let output = Command::new("dnf")
            .args(["search", query])
            .output()
            .ok();

        // Parse output and return Vec<Package>.
        // Use parse_alternating_lines if the format fits,
        // or write a custom parser.
        todo!()
    }

    fn get_installed(&self) -> HashSet<String> {
        let output = Command::new("dnf")
            .args(["list", "--installed"])
            .output()
            .ok();
        // Parse and return package names.
        todo!()
    }

    fn get_installed_details(&self) -> Vec<Package> {
        todo!()
    }

    fn get_updates(&self) -> Vec<Package> {
        let output = Command::new("dnf")
            .args(["list", "--upgrades"])
            .output()
            .ok();
        todo!()
    }

    fn get_details(&self, pkg: &str, _provider: &str) -> Option<HashMap<String, String>> {
        // Check DETAILS_CACHE first.
        {
            let cache = crate::managers::DETAILS_CACHE.lock().unwrap();
            if let Some(cached) = cache.get(pkg) {
                return Some(cached.clone());
            }
        }

        let output = Command::new("dnf")
            .args(["info", pkg])
            .output()
            .ok()?;

        // Parse colon-separated key: value lines and store in DETAILS_CACHE.
        todo!()
    }

    fn install(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let names: Vec<&str> = pkgs.iter().map(String::as_str).collect();
        let mut args = vec!["dnf", "install"];
        args.extend(names);
        crate::execute_external_command(terminal, "sudo", &args)
    }

    fn remove(
        &self,
        terminal: &mut DefaultTerminal,
        pkgs: &HashSet<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let names: Vec<&str> = pkgs.iter().map(String::as_str).collect();
        let mut args = vec!["dnf", "remove"];
        args.extend(names);
        crate::execute_external_command(terminal, "sudo", &args)
    }

    fn system_upgrade(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "sudo", &["dnf", "upgrade"])
    }

    fn refresh_databases(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::execute_external_command(terminal, "sudo", &["dnf", "check-update"])
    }
}
```

---

## Step 3 — Register the backend

In `src/managers/mod.rs`, add a detection branch in `get_system_manager`:

```rust
pub fn get_system_manager(config: &crate::config::Config) -> Box<dyn PackageManager> {
    if std::env::consts::OS == "macos" {
        return Box::new(brew::BrewManager);
    }

    if std::process::Command::new("pacman").arg("--version").output().is_ok() {
        return Box::new(arch::ArchManager::new(config.aur_helper.clone()));
    }

    if std::process::Command::new("apt").arg("--version").output().is_ok() {
        return Box::new(apt::AptManager);
    }

    // Add your backend here:
    if std::process::Command::new("dnf").arg("--version").output().is_ok() {
        return Box::new(dnf::DnfManager);
    }

    Box::new(arch::ArchManager::new(config.aur_helper.clone()))
}
```

---

## Tips

- **Use `parse_alternating_lines`** if the package manager outputs results in the standard `name version\n  description` format.
- **Always check `DETAILS_CACHE`** at the top of `get_details` to avoid redundant subprocess calls.
- **Call `execute_external_command`** for any operation that produces interactive output (confirmations, progress bars, etc.).
- **Make the struct `Send + Sync`** — the `PackageManager` trait bound requires it. Zero-size structs and structs with only owned data satisfy this automatically.
- Run `cargo clippy` before opening a PR — Clippy warnings should ideally be zero.
