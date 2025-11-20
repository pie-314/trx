Thank you for your interest in contributing to **TRX**.
This document outlines the guidelines for contributing code, documentation, and ideas to the project.

---

# **1. Getting Started**

### **Clone the repository**

```bash
git clone https://github.com/pie-314/trx.git
cd trx
```

### **Build the project**

```bash
cargo build
```

### **Run the project**

```bash
cargo run
```

### **Run tests**

```bash
cargo test
```

Ensure the project builds without warnings before opening a pull request.

---

# **2. Project Structure**

```
src/
├── main.rs          # Application entry point
├── app.rs           # Global state + reducer
├── ui/              # Terminal UI components
├── managers/        # Package manager backends
└── fuzzy/           # Fuzzy matching engine
```

---

# **3. Contribution Areas**

You can contribute in any of these domains:

### **Backend Integrations**

Implement or improve package manager providers (apt, dnf, brew, winget, etc.).

### **TUI Improvements**

Optimizing rendering, new widgets, theme system, layout work.

### **Fuzzy Search Engine**

Improving scoring, heuristics, performance, or incremental updates.

### **Performance Work**

Caching, async pipelines, parallel execution.

### **Bug Fixes**

Reproduce, isolate, and fix issues.

### **Documentation**

Improve README, examples, architecture docs, or this file.

---

# **4. Coding Guidelines**

### **General**

* Keep the codebase clean and idiomatic.
* Prefer pure functions in UI components.
* Avoid blocking I/O on the main/UI thread.
* Use `tokio` for all asynchronous operations.
* Return structured errors using `ManagerError`.

### **Style**

* Follow standard Rust formatting:

```bash
cargo fmt
cargo clippy
```

(Clippy warnings should ideally be zero.)

### **Commits**

* Use clear, atomic commit messages.
* Example:

  * `feat(pacman): async metadata caching`
  * `fix(ui): prevent double redraw on search`
  * `docs: update usage section`

---

# **5. Issues & Pull Requests**

### **Issues**

Before creating an issue:

* Check if the issue already exists.
* Provide steps to reproduce.
* Include logs, platform, and version when relevant.

Labels you will commonly see:

* `good first issue`
* `help wanted`
* `backend`
* `tui`
* `fuzzy`
* `performance`

### **Pull Requests**

1. Fork the repository
2. Create a feature branch

   ```bash
   git checkout -b feature/my-improvement
   ```
3. Make your changes
4. Run tests + lint
5. Commit and push
6. Open a PR describing:

   * what changed
   * why
   * how it was tested

PRs should not include unrelated formatting changes.

---

# **6. Adding a New Package Manager Backend**

To add a new provider, implement the trait:

```rust
pub trait PackageManager {
    fn search(&self, query: &str) -> Result<Vec<Package>, ManagerError>;
    fn install(&self, pkg: &str) -> Result<(), ManagerError>;
    fn remove(&self, pkg: &str) -> Result<(), ManagerError>;
    fn update(&self, pkg: &str) -> Result<(), ManagerError>;
    fn info(&self, pkg: &str) -> Result<PackageInfo, ManagerError>;
}
```

Refer to existing backends (pacman/yay) for structure and system-call patterns.

---

# **7. Development Environment**

Recommended tools:

* Rust 1.70+
* Clippy (`rustup component add clippy`)
* Rustfmt (`rustup component add rustfmt`)
* A terminal supporting Unicode + truecolor

Optional tools:

* `cargo-expand` for macro debugging
* `cargo-insta` for snapshot testing (planned)
* `just` for project scripts (future integration)

---

# **8. Code of Conduct**

All discussions and contributions must follow respectful, constructive communication.
Harassment, discrimination, or hostile behavior is not tolerated.

---

# **9. Questions / Discussions**

Use **GitHub Issues** or **Discussions** for:

* feature proposals
* design debates
* questions about architecture
* help with contributing

---

# **10. Thank You**

Your contributions make TRX better.
Even small improvements—typos, refactors, documentation—are highly valued.

