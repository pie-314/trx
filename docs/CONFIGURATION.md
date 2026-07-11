# trx Configuration Guide

`trx` can be customized through a `config.toml` file. This file is automatically created with default values the first time you run the application.

## Configuration File Location

Depending on your operating system, the configuration file is located at:

- **Linux:** `~/.config/trx/config.toml`
- **macOS:** `~/Library/Application Support/trx/config.toml`

---

## Settings (`[settings]`)

General behavioral settings for the application.

| Option | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `aur_helper` | String | `"yay"` | The AUR helper to use for Arch Linux (e.g., `yay`, `paru`). |
| `search_debounce_ms` | Integer | `200` | Delay in milliseconds before a search is triggered while typing. |
| `auto_update_check` | Boolean | `true` | Whether to check for `trx` updates on startup. |
| `default_tab` | String | `"Search"` | The tab to display on startup (`"Search"`, `"Installed"`, or `"Updates"`). |
| `max_search_results` | Integer | `50` | Maximum number of results to display in the search list. |
| `sandbox` | Boolean | `false` | Linux-only opt-in Bubblewrap sandbox for install/remove subprocesses. Falls back with a warning if `bwrap` is unavailable. |

---

## Keybindings (`[keys]`)

Customize the keyboard shortcuts for various actions. Most keys are single characters.

| Action | Default | Description |
| :--- | :--- | :--- |
| `quit` | `"q"` | Exit the application. |
| `install` | `"i"` | Install selected packages. |
| `remove` | `"x"` | Remove selected packages. |
| `search_edit` | `"e"` | Enter editing mode for the search bar. |
| `tab_next` | `"Tab"` | Switch to the next tab. |
| `tab_prev` | `"BackTab"` | Switch to the previous tab. |
| `toggle_select` | `" "` | Select or deselect the highlighted package (Space). |
| `system_upgrade` | `"U"` | Perform a full system upgrade. |
| `refresh_db` | `"R"` | Refresh package manager databases. |
| `help` | `"?"` | Toggle the help overlay. |

---

## Theme (`[theme]`)

Customize the visual appearance of the TUI. Colors can be standard names like `"blue"`, `"green"`, `"red"`, `"yellow"`, `"magenta"`, `"cyan"`, `"white"`, `"black"`, or their "light" variants (e.g., `"light_blue"`).

| Option | Default | Description |
| :--- | :--- | :--- |
| `border_color` | `"blue"` | Color of the UI component borders. |
| `highlight_color` | `"yellow"` | Color of the selected tab and active pointer. |
| `success_color` | `"green"` | Color for installed indicators and positive actions. |
| `error_color` | `"red"` | Color for error messages and destructive actions. |
| `text_primary` | `"white"` | Main text color. |
| `text_secondary` | `"cyan"` | Color for secondary info like package providers. |

---

## Full Example

```toml
aur_helper = "paru"

[settings]
search_debounce_ms = 300
auto_update_check = true
default_tab = "Search"
max_search_results = 100
sandbox = false

[keys]
quit = "q"
install = "i"
remove = "r"
search_edit = "/"
toggle_select = " "

[theme]
border_color = "magenta"
highlight_color = "cyan"
success_color = "green"
error_color = "red"
text_primary = "white"
text_secondary = "light_blue"
```
