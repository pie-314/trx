# Coding Guidelines

## Code Style

TRX follows standard Rust idioms.

- **No AI Noise:** Do not add useless, redundant, or AI-generated comments that just restate what the code does.
- **Respect Context:** Do not delete existing comments unless the associated code is removed or the comment is objectively wrong.
- **Formatting:** Before committing, always run:

```bash
cargo fmt
cargo clippy
```

Clippy warnings should ideally be zero. Rustfmt is non-negotiable.

---

## Architecture Principles

- **Keep the UI thread non-blocking.** Never call blocking I/O on the main loop thread. Use `std::thread::spawn` + `mpsc` channels for any subprocess call.
- **Prefer OS threads over async.** TRX intentionally does not use Tokio or async/await. New features should follow the existing `std::thread` + `mpsc` pattern.
- **Return structured errors.** Use `Box<dyn std::error::Error>` for backend method errors, matching the trait signature. Do not panic on expected failure paths.
- **Cache detail lookups.** Any `get_details` implementation should check and populate `DETAILS_CACHE` to avoid redundant subprocess calls.
- **Pure rendering.** The `draw_ui` function and its helpers should read state but not mutate business logic. Keep rendering predictable.

---

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(dnf): implement DnfManager backend
fix(ui): prevent double redraw on search
docs: update architecture overview
refactor(fuzzy): simplify gap penalty calculation
```

Scope tokens: `ui`, `fuzzy`, `pacman`, `apt`, `brew`, `aur`, `config`, `updater`, `docs`.

---

## Development Tools

| Tool | Install | Purpose |
|------|---------|---------|
| `rustfmt` | `rustup component add rustfmt` | Code formatting |
| `clippy` | `rustup component add clippy` | Linting |
| `cargo-expand` | `cargo install cargo-expand` | Macro debugging |

---

## Environment Requirements

- Rust **1.70+**
- A terminal supporting **Unicode** and **truecolor**
- The package manager for your platform installed and on `$PATH`
