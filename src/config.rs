use directories::ProjectDirs;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub search_debounce_ms: u64,
    pub auto_update_check: bool,
    pub auto_cleanup: bool,
    pub default_tab: String,
    pub max_search_results: usize,
    pub enabled_managers: Vec<String>,
    pub border_style: String, // "Plain", "Rounded", "Double", "Thick"
    pub spinner_type: String, // "Dots", "Bars", "Pulse", "Classic", "Arc", "Braille"
    /// Version the user explicitly skipped. Ignored while the same tag is
    /// latest; cleared from config when a genuinely newer release is detected,
    /// so the prompt resurfaces automatically for new updates.
    pub skipped_update_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Keys {
    pub quit: String,
    pub install: String,
    pub remove: String,
    pub update: String,
    pub search_edit: String,
    pub toggle_select: String,
    pub tab_next: String,
    pub tab_prev: String,
    pub system_upgrade: String,
    pub refresh_db: String,
    pub help: String,
    #[serde(default = "default_check_update_key")]
    pub check_update: String,
}

fn default_check_update_key() -> String {
    "C".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Theme {
    pub border_color: String,
    pub highlight_color: String,
    pub success_color: String,
    pub error_color: String,
    pub text_primary: String,
    pub text_secondary: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub aur_helper: String,
    pub settings: Settings,
    pub keys: Keys,
    pub theme: Theme,
    pub theme_name: String,
    pub custom_theme: Option<Theme>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aur_helper: "yay".to_string(),
            settings: Settings {
                search_debounce_ms: 200,
                auto_update_check: true,
                auto_cleanup: false,
                default_tab: "Search".to_string(),
                max_search_results: 50,
                enabled_managers: vec![
                    "pacman".to_string(),
                    "yay".to_string(),
                    "brew".to_string(),
                    "apt".to_string(),
                ],
                border_style: "Rounded".to_string(),
                spinner_type: "Dots".to_string(),
                skipped_update_version: None,
            },
            keys: Keys {
                quit: "q".to_string(),
                install: "i".to_string(),
                remove: "x".to_string(),
                update: "u".to_string(),
                search_edit: "e".to_string(),
                toggle_select: " ".to_string(),
                tab_next: "Tab".to_string(),
                tab_prev: "BackTab".to_string(),
                system_upgrade: "U".to_string(),
                refresh_db: "R".to_string(),
                help: "?".to_string(),
                check_update: "C".to_string(),
            },
            theme: Theme {
                border_color: "blue".to_string(),
                highlight_color: "yellow".to_string(),
                success_color: "green".to_string(),
                error_color: "red".to_string(),
                text_primary: "white".to_string(),
                text_secondary: "cyan".to_string(),
            },
            theme_name: "Default".to_string(),
            custom_theme: None,
        }
    }
}

impl Keys {
    /// Mutates a keybinding string field inside the struct dynamically based on its text action name.
    pub fn update_action_key(&mut self, action_name: &str, new_key_str: String) {
        match action_name {
            "quit" => self.quit = new_key_str,
            "install" => self.install = new_key_str,
            "remove" => self.remove = new_key_str,
            "update" => self.update = new_key_str,
            "search_edit" => self.search_edit = new_key_str,
            "toggle_select" => self.toggle_select = new_key_str,
            "tab_next" => self.tab_next = new_key_str,
            "tab_prev" => self.tab_prev = new_key_str,
            "system_upgrade" => self.system_upgrade = new_key_str,
            "refresh_db" => self.refresh_db = new_key_str,
            "help" => self.help = new_key_str,
            "check_update" => self.check_update = new_key_str,
            _ => {}
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "trx") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.toml");

            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(config_path)
                    && let Ok(config) = toml::from_str(&content)
                {
                    return config;
                }
            } else {
                // Create default config
                let _ = fs::create_dir_all(config_dir);
                let _ = fs::write(&config_path, toml::to_string(&Self::default()).unwrap());
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "trx") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.toml");
            fs::create_dir_all(config_dir)?;
            let content = toml::to_string(self)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }

    pub fn get_color(&self, color_name: &str) -> Color {
        if color_name.starts_with('#') && color_name.len() == 7 {
            let r = u8::from_str_radix(&color_name[1..3], 16).unwrap_or(0);
            let g = u8::from_str_radix(&color_name[3..5], 16).unwrap_or(0);
            let b = u8::from_str_radix(&color_name[5..7], 16).unwrap_or(0);
            return Color::Rgb(r, g, b);
        }

        match color_name.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            "light_red" => Color::LightRed,
            "light_green" => Color::LightGreen,
            "light_yellow" => Color::LightYellow,
            "light_blue" => Color::LightBlue,
            "light_magenta" => Color::LightMagenta,
            "light_cyan" => Color::LightCyan,
            "white" => Color::White,
            _ => Color::Reset,
        }
    }

    /// Returns a high-contrast foreground colour (black or white) suitable for
    /// text rendered on top of `bg`, determined by the perceived luminance of the
    /// background (W3C relative luminance formula, simplified sRGB linearisation).
    pub fn contrast_fg_for(bg: Color) -> Color {
        let (r, g, b) = match bg {
            Color::Rgb(r, g, b) => (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0),
            Color::Black | Color::DarkGray => return Color::White,
            Color::White | Color::Gray => return Color::Black,
            // For named colours, fall back to dark fg (most themes use light bg highlights).
            _ => return Color::Black,
        };
        // Linearise each sRGB channel and compute relative luminance (Y).
        let linearise = |c: f64| -> f64 {
            if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
        };
        let y = 0.2126 * linearise(r) + 0.7152 * linearise(g) + 0.0722 * linearise(b);
        if y >= 0.179 { Color::Black } else { Color::White }
    }

    pub fn current_theme(&self) -> Theme {
        if self.theme_name == "Custom"
            && let Some(ref custom) = self.custom_theme
        {
            return custom.clone();
        }

        match self.theme_name.as_str() {
            "Nord" => Theme {
                border_color: "#81A1C1".to_string(),
                highlight_color: "#88C0D0".to_string(),
                success_color: "#A3BE8C".to_string(),
                error_color: "#BF616A".to_string(),
                text_primary: "#ECEFF4".to_string(),
                text_secondary: "#D8DEE9".to_string(),
            },
            "Dracula" => Theme {
                border_color: "#6272a4".to_string(),
                highlight_color: "#ff79c6".to_string(),
                success_color: "#50fa7b".to_string(),
                error_color: "#ff5555".to_string(),
                text_primary: "#f8f8f2".to_string(),
                text_secondary: "#bd93f9".to_string(),
            },
            "OneDark" => Theme {
                border_color: "#61afef".to_string(),
                highlight_color: "#c678dd".to_string(),
                success_color: "#98c379".to_string(),
                error_color: "#e06c75".to_string(),
                text_primary: "#abb2bf".to_string(),
                text_secondary: "#d19a66".to_string(),
            },
            "Gruvbox" => Theme {
                border_color: "#fabd2f".to_string(),
                highlight_color: "#83a598".to_string(),
                success_color: "#b8bb26".to_string(),
                error_color: "#fb4934".to_string(),
                text_primary: "#ebdbb2".to_string(),
                text_secondary: "#d5c4a1".to_string(),
            },
            "Solarized" => Theme {
                border_color: "#268bd2".to_string(),
                highlight_color: "#d33682".to_string(),
                success_color: "#859900".to_string(),
                error_color: "#dc322f".to_string(),
                text_primary: "#93a1a1".to_string(),
                text_secondary: "#839496".to_string(),
            },
            _ => self.theme.clone(),
        }
    }
}
