# Contributing Guide

Contributions to TRX are welcome — whether that's a bug fix, a new backend, a UI improvement, or documentation. This page provides an overview. For full details, see [CONTRIBUTING.md](https://github.com/pie-314/trx/blob/main/CONTRIBUTING.md) in the repository.

---

## Getting Started

```bash
git clone https://github.com/pie-314/trx.git
cd trx
cargo build        # ensure the project compiles
cargo run          # smoke test in your terminal
cargo test         # run the test suite
```

---

## Contribution Areas

| Area | Examples |
|------|---------|
| **Backend Integrations** | New package managers (dnf, zypper, winget) |
| **TUI Improvements** | New widgets, themes, layout changes |
| **Fuzzy Search** | Better scoring heuristics, performance |
| **Performance** | Caching, parallel execution |
| **Bug Fixes** | Reproduce, isolate, and fix issues |
| **Documentation** | Improve this site, README, examples |

---

## Pull Request Workflow

1. Fork the repository and create a feature branch (target the `dev` branch):
   ```bash
   git checkout dev
   git checkout -b feat/my-improvement
   ```
2. Make your changes.
3. Run `cargo fmt` and `cargo clippy` (zero warnings preferred).
4. Run `cargo test`.
5. Commit using conventional commit messages (see [Coding Guidelines](./coding-guidelines.md)).
6. Open a PR **against the `dev` branch** describing what changed, why, and how it was tested.

---

## Issues

Before filing an issue, check if it already exists. When reporting a bug, include:

- Steps to reproduce
- Platform (OS, package manager version)
- TRX version (`trx --version`)
- Relevant terminal output or screenshots

Common issue labels: `good first issue`, `help wanted`, `backend`, `tui`, `fuzzy`, `performance`.
