# Roadmap

This page tracks planned and in-progress features for TRX.

---

## In Progress / Near-term

| Feature | Description |
|---------|-------------|
| **dnf / yum backend** | Fedora and RHEL support via `dnf` |
| **zypper backend** | openSUSE support |
| **winget / scoop backend** | Windows support |

---

## Planned

| Feature | Description |
|---------|-------------|
| **Transaction history** | Log of installs/removes with rollback support |
| **Batch / scripting mode** | Non-interactive mode for CI and shell scripts |
| **Dependency graph visualiser** | Visual view of package dependency trees |
| **Metadata caching** | Persist `DETAILS_CACHE` across sessions for faster repeated searches |
| **Plugin system** | Load custom backends and widgets from shared libraries |

---

## Completed

| Feature | Version |
|---------|---------|
| Pacman backend | v0.1.0 |
| AUR (yay) backend | v0.1.0 |
| APT backend | v0.1.0 |
| Homebrew backend | v0.1.0 |
| Self-updating mechanism | v0.1.2 |
| Binary releases via GitHub Actions | v0.1.4 |
| Configurable keybindings | v0.1.5 |
| Built-in and Custom Themes | v0.1.6 |
| Mouse Support | v0.1.7 |
| Settings Tab | v0.1.8 |

---

## Contributing to the Roadmap

If you want to work on a planned feature:
1. Open a GitHub Discussion or comment on the relevant issue.
2. **Describe how you plan to implement the feature.**
3. **Provide mockups or screenshots** if it involves UI changes.
4. Wait for assignment before starting work.

See the [Contributing Guide](./contributing/index.md) for more details.
