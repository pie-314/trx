<div align="center">

<img src="assets/logo.svg" width="180" alt="TRX" />

<br/>
<br/>

**A fast, keyboard-driven TUI package manager written in Rust.**

Search 50,000+ packages in under 50ms. Install, remove, and update without leaving your terminal.

<br/>

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?style=flat&logo=rust)
![License](https://img.shields.io/badge/License-MIT-green?style=flat)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Arch%20%7C%20Debian-blue?style=flat)

</div>

<details>
<summary>📑 Table of Contents</summary>

- [What is TRX?](#what-is-trx)
- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Keybindings](#keybindings)
- [Architecture](#architecture)
- [Supported Package Managers](#supported-package-managers)
- [Configuration](#configuration)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

</details>

---

<div align="center">

<img src="assets/trx-preview.gif" alt="TRX in action" width="720" />

</div>

---

## What is TRX?

TRX is a terminal UI built on top of your existing package manager. It gives you a unified, keyboard-first interface for searching, inspecting, and managing packages, whether you're on macOS with Homebrew, Arch with Pacman + AUR, or Debian/Ubuntu with APT.

No daemon required. Fully configurable via `config.toml` or the in-app Settings tab.

---

## Features

* Renderer built on `ratatui` with deterministic layout and minimal redraw overhead  
* Fully non-blocking event loop using **OS threads** and **mpsc channels** (no async runtime overhead)
* Unified command model for package managers with pluggable backend architecture  
* **Tiered fuzzy search** — five-tier scorer: exact → prefix → word-boundary → consecutive → subsequence; results ranked by relevance
* **Install and remove packages** without leaving the TUI (`i`, `x` keys)  
* **Settings & Themes** – Configure keybindings, themes (Nord, Dracula, etc.), and UI styles in-app
* **Mouse Support** – Full navigation and interaction via mouse
* **Self-updating mechanism** – Checks for new releases on startup and updates automatically
* **Search result caching** – Repeat queries are served instantly from an in-memory cache
* Stateless backend operations executed via system calls with structured output parsing  
* Extensible design suitable for adding new package managers without modifying the core engine  

---

## Installation

### One-liner (Recommended)

```bash
curl -fsSL https://trx.pidev.tech/install.sh | sh
```

### From source

```bash
git clone https://github.com/pie-314/trx.git
cd trx
cargo build --release
sudo cp target/release/trx /usr/local/bin/
```

**Requirements:** Rust 1.70+ · Unicode/truecolor terminal

### Cargo

```bash
cargo install trx-cli
```

---

## Usage

```bash
trx
```

### Keybindings

| Key | Action |
|-----|--------|
| `e` | Enter search mode |
| `↑` / `↓` or `j` / `k` | Navigate list |
| `space` | Toggle package selection |
| `i` | Install selected package(s) |
| `x` | Remove selected package(s) |
| `U` | System-wide upgrade |
| `R` | Refresh package databases |
| `Tab` | Switch tab (Search → Installed → Updates → Settings) |
| `?` | Toggle help overlay (quick keybinding reference) |
| `q` / `Esc` | Quit / exit current mode |

> All keybindings are configurable in `~/.config/trx/config.toml`.

---

## Architecture

```
src/
├── main.rs              # Entry point, terminal setup
├── config.rs            # Configuration and Theme management
├── updater.rs           # Self-update logic
├── ui/
│   ├── app.rs           # App state, event loop, channel polling
│   ├── draw.rs          # UI rendering and layout
│   └── input.rs         # Input mode handling
├── managers/
│   ├── mod.rs           # PackageManager trait, shared parsing
│   ├── arch.rs          # Parallel Pacman + AUR (RPC API) backend
│   ├── apt.rs           # APT backend
│   └── brew.rs          # Homebrew backend
└── fuzzy/
    └── mod.rs           # Substring-optimized scoring engine
```

### PackageManager trait

```rust
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn search(&self, query: &str) -> Vec<Package>;
    fn get_installed(&self) -> HashSet<String>;
    fn get_installed_details(&self) -> Vec<Package>;
    fn get_updates(&self) -> Vec<Package>;
    fn get_details(&self, pkg: &str, provider: &str) -> Option<HashMap<String, String>>;
    fn install(&self, terminal: &mut DefaultTerminal, pkgs: &HashSet<String>) -> Result<(), Box<dyn std::error::Error>>;
    fn remove(&self, terminal: &mut DefaultTerminal, pkgs: &HashSet<String>) -> Result<(), Box<dyn std::error::Error>>;
    fn system_upgrade(&self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>>;
    fn refresh_databases(&self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn std::error::Error>>;
}
```

### Concurrency model

Search, list loads, and detail fetches all run on **OS threads** communicating via `std::sync::mpsc`. The main loop polls keyboard/mouse input with a short timeout, redraws each frame, and non-blockingly drains result channels. Large searches (like Arch + AUR) are parallelized using **Rayon**.

---

## Supported Package Managers

| Manager | Platform | Status |
|---------|----------|--------|
| Pacman | Arch Linux | ✅ Implemented |
| yay (AUR) | Arch Linux | ✅ Implemented |
| APT | Debian / Ubuntu | ✅ Implemented |
| Homebrew | macOS / Linux | ✅ Implemented |
| dnf / yum | Fedora / RHEL | 🔜 Planned |
| zypper | openSUSE | 🔜 Planned |
| winget / scoop | Windows | 🔜 Planned |

---

## Configuration

TRX stores its config at `~/.config/trx/config.toml` (created automatically on first run).

```toml
aur_helper = "yay"          # AUR helper used for AUR installs/removals (yay, paru, etc.)
theme_name = "Default"      # Default, Nord, Dracula, OneDark, Gruvbox, Solarized, Custom

[settings]
search_debounce_ms = 200    # Delay (ms) before triggering search after typing stops
max_search_results = 50     # Maximum number of packages shown in search results
auto_update_check = true    # Check for TRX updates on startup
auto_cleanup = false        # Remove unused package caches after operations
default_tab = "Search"      # Starting tab: Search, Installed, Updates, Settings
border_style = "Rounded"    # Border style: Plain, Rounded, Double, Thick
spinner_type = "Dots"       # Loading spinner: Dots, Bars, Pulse, Classic
enabled_managers = ["pacman", "yay", "brew", "apt"]
skipped_update_version = "" # Version string that the user chose to skip (managed automatically)

[keys]
quit = "q"
install = "i"
remove = "x"
search_edit = "e"
toggle_select = " "
tab_next = "Tab"
tab_prev = "BackTab"
system_upgrade = "U"
refresh_db = "R"
help = "?"

# The `theme` block sets the Default preset colors; `custom_theme` is used
# when theme_name = "Custom".  Only one of the two is active at a time.
[theme]
border_color = "blue"
highlight_color = "yellow"
success_color = "green"
error_color = "red"
text_primary = "white"
text_secondary = "cyan"
# Colors can be named ("blue", "cyan", etc.) or hex ("#81A1C1")
```

---

## Roadmap

- [x] Configurable keybindings via config file
- [x] Pluggable themes and renderer settings
- [x] Settings Tab for in-app configuration
- [x] Mouse support
- [ ] Package update (per-package, not just system upgrade)
- [x] Search result caching for repeated queries
- [x] AUR search via RPC API (no yay dependency for search)
- [ ] Transaction history and rollback
- [ ] Batch mode for scripting / CI use
- [ ] Dependency graph visualizer
- [ ] Plugin system for custom backends and widgets
- [x] Binary releases via GitHub Actions

---

## Contributing

Contributions are welcome. If you're interested in Rust, terminal UIs, or package manager internals, pick an open issue or start a discussion.

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT. See [LICENSE](LICENSE) for details.

/**
## ✨ README Improvement Notes

### 📌 Formatting Enhancements Needed
- Improve heading hierarchy for better readability
- Ensure consistent spacing between sections
- Use proper Markdown formatting for code blocks and lists
- Align all installation and usage steps properly

### 🚀 Suggested Structure Upgrade
- Introduction
- Features
- Tech Stack
- Installation
- Usage
- Project Structure
- Contribution Guidelines
- License

### 🛠️ Documentation Improvements
- Add badges (optional): build, license, contributors
- Add screenshots for better UI understanding
- Standardize code blocks for commands

### 🎯 Goal
Improve onboarding experience for new contributors and users by making README more structured, readable, and professional.
*/
