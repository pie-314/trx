use crate::ui::{draw::draw_ui, input::InputMode};

use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::ListState,
    style::Color,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::managers::{self, Package};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Search,
    Installed,
    Updates,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DetailsState {
    Empty,
    Loading,
    Success(std::collections::HashMap<String, String>),
    Error(String),
}

pub struct App {
    pub input: String,
    pub character_index: usize,
    pub input_mode: InputMode,
    pub current_tab: Tab,
    pub packages: Vec<Package>,
    pub checked: Vec<bool>,
    pub selected_names: HashSet<String>,
    pub installed_packages: HashSet<String>,
    pub selected: usize,
    pub list_state: ListState,
    pub messages: Vec<String>,
    pub loading: bool,
    pub details_state: DetailsState,
    pub last_selected: usize,
    pub show_help: bool,
    pub update_prompt: Option<(String, String)>, // (version, url)
    pub update_selected_yes: bool,
    pub spinner_tick: u8,
    pub manager: Arc<Box<dyn managers::PackageManager>>,
    pub config: crate::config::Config,
    pub settings_index: usize,
    pub details_scroll: u16,
    pub available_managers: Vec<String>,
    pub popup_message: Option<(String, Color)>, // (message, color)
    result_tx: Sender<(String, Vec<Package>)>,
    result_rx: Receiver<(String, Vec<Package>)>,
    details_tx: Sender<DetailsState>,
    details_rx: Receiver<DetailsState>,
    update_rx: Receiver<Option<(String, String)>>,
    last_input_time: Instant,
    pending_search: bool,
    pub last_search_query: String,
    pub popup_timer: Option<Instant>,
}

impl App {
    pub fn new(result_tx: Sender<(String, Vec<Package>)>, result_rx: Receiver<(String, Vec<Package>)>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(None);

        let (details_tx, details_rx) = std::sync::mpsc::channel();
        let (update_tx, update_rx) = std::sync::mpsc::channel();
        let config = crate::config::Config::load();
        
        // Spawn parallel update check if enabled
        if config.settings.auto_update_check {
            let skipped = config.settings.skipped_update_version.clone();
            thread::spawn(move || {
                let res = crate::updater::check_for_updates(skipped.as_deref());
                let _ = update_tx.send(res);
            });
        }

        let manager = Arc::new(managers::get_system_manager(&config));
        let available_managers = managers::get_available_managers();

        let current_tab = match config.settings.default_tab.as_str() {
            "Installed" => Tab::Installed,
            "Updates" => Tab::Updates,
            _ => Tab::Search,
        };

        let mut app = Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            current_tab,
            packages: Vec::new(),
            checked: Vec::new(),
            selected_names: HashSet::new(),
            installed_packages: manager.get_installed(),
            selected: 0,
            list_state,
            messages: Vec::new(),
            loading: false,
            details_state: DetailsState::Empty,
            last_selected: usize::MAX,
            show_help: false,
            update_prompt: None,
            update_selected_yes: true,
            spinner_tick: 0,
            manager,
            config,
            settings_index: 0,
            details_scroll: 0,
            character_index: 0,
            available_managers,
            popup_message: None,
            popup_timer: None,
            result_tx,
            result_rx,
            details_tx,
            details_rx,
            update_rx,
            last_input_time: Instant::now(),
            pending_search: false,
            last_search_query: String::new(),
        };

        if app.current_tab != Tab::Search {
            app.reset_tab_state();
        }

        app
    }

    pub fn set_popup(&mut self, msg: String, color: Color) {
        self.popup_message = Some((msg, color));
        self.popup_timer = Some(Instant::now());
    }

    fn move_cursor_left(&mut self) {
        let new_index = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(new_index);
    }

    fn move_cursor_right(&mut self) {
        let new_index = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(new_index);
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn clamp_cursor(&self, new_pos: usize) -> usize {
        new_pos.clamp(0, self.input.chars().count())
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();

        self.last_input_time = Instant::now();
        self.pending_search = true;
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let left = self.character_index - 1;
            let before = self.input.chars().take(left);
            let after = self.input.chars().skip(self.character_index);
            self.input = before.chain(after).collect();
            self.move_cursor_left();

            self.last_input_time = Instant::now();
            self.pending_search = true;
        }
    }

    fn check_and_execute_search(&mut self) {
        let debounce_ms = self.config.settings.search_debounce_ms;

        if self.pending_search
            && self.last_input_time.elapsed() >= Duration::from_millis(debounce_ms)
        {
            let query = self.input.trim().to_string();

            if !query.is_empty() && query != self.last_search_query {
                self.last_search_query = query.clone();
                self.pending_search = false;
                self.loading = true;

                let tx = self.result_tx.clone();
                let manager = self.manager.clone();
                let q_clone = query.clone();

                thread::spawn(move || {
                    let all = manager.search(&q_clone);
                    let _ = tx.send((q_clone, all));
                });
            } else if query.is_empty() {
                self.pending_search = false;
                self.packages.clear();
                self.messages.clear();
                self.loading = false;
            }
        }
    }
    fn run_command(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.selected_names.is_empty() {
            return Ok(());
        }

        self.manager.install(terminal, &self.selected_names)
    }

    fn run_remove_command(
        &self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.selected_names.is_empty() {
            return Ok(());
        }

        let mut to_remove = HashSet::new();
        for name in &self.selected_names {
            if self.installed_packages.contains(name) {
                to_remove.insert(name.clone());
            }
        }

        if !to_remove.is_empty() {
            self.manager.remove(terminal, &to_remove)?;
        }

        Ok(())
    }

    fn switch_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Search => Tab::Installed,
            Tab::Installed => Tab::Updates,
            Tab::Updates => Tab::Settings,
            Tab::Settings => Tab::Search,
        };

        self.reset_tab_state();
    }

    fn switch_tab_previous(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Search => Tab::Settings,
            Tab::Installed => Tab::Search,
            Tab::Updates => Tab::Installed,
            Tab::Settings => Tab::Updates,
        };

        self.reset_tab_state();
    }

    fn reset_tab_state(&mut self) {
        self.packages.clear();
        self.messages.clear();
        self.checked.clear();
        self.last_search_query.clear();
        self.selected = 0;
        self.list_state.select(None);
        self.details_state = DetailsState::Empty;
        self.details_scroll = 0;

        match self.current_tab {
            Tab::Search => {
                self.pending_search = true;
            }
            Tab::Installed => {
                self.loading = true;
                let tx = self.result_tx.clone();
                let manager = self.manager.clone();
                thread::spawn(move || {
                    let pkgs = manager.get_installed_details();
                    let _ = tx.send(("__INSTALLED__".to_string(), pkgs));
                });
            }
            Tab::Updates => {
                self.loading = true;
                let tx = self.result_tx.clone();
                let manager = self.manager.clone();
                thread::spawn(move || {
                    let pkgs = manager.get_updates();
                    let _ = tx.send(("__UPDATES__".to_string(), pkgs));
                });
            }
            Tab::Settings => {
                self.loading = false;
            }
        }
    }

    fn trigger_details_fetch(&mut self) {
        if self.packages.is_empty() || self.selected >= self.packages.len() {
            self.details_state = DetailsState::Empty;
            return;
        }

        if self.selected == self.last_selected {
            return;
        }

        let pkg = self.packages[self.selected].clone();
        let tx = self.details_tx.clone();
        let manager = self.manager.clone();
        self.last_selected = self.selected;
        self.details_state = DetailsState::Loading;
        self.details_scroll = 0;

        thread::spawn(move || {
            let info = manager.get_details(&pkg.name, &pkg.provider);
            if let Some(details) = info {
                let _ = tx.send(DetailsState::Success(details));
            } else {
                let _ = tx.send(DetailsState::Error("Failed to fetch details".to_string()));
            }
        });
    }

    fn toggle_manager(&mut self, name: &str) {
        if self.config.settings.enabled_managers.contains(&name.to_string()) {
            if self.config.settings.enabled_managers.len() > 1 {
                self.config.settings.enabled_managers.retain(|m| m != name);
                self.set_popup(format!("Disabled {}", name), Color::Yellow);
            } else {
                self.set_popup("Must have at least one manager enabled".to_string(), Color::Red);
            }
        } else {
            self.config.settings.enabled_managers.push(name.to_string());
            self.set_popup(format!("Enabled {}", name), Color::Green);
        }
        let _ = self.config.save();
        // Re-initialize manager
        self.manager = Arc::new(managers::get_system_manager(&self.config));
        
        // Refresh current list if applicable
        if self.current_tab != Tab::Settings {
            self.reset_tab_state();
        }
    }

    fn next_theme(&mut self) {
        let themes = vec!["Default", "Nord", "Dracula", "OneDark", "Gruvbox", "Solarized", "Custom"];
        let current_pos = themes.iter().position(|&t| t == self.config.theme_name).unwrap_or(0);
        let next_pos = (current_pos + 1) % themes.len();
        self.config.theme_name = themes[next_pos].to_string();
        if self.config.theme_name == "Custom" && self.config.custom_theme.is_none() {
            self.config.custom_theme = Some(self.config.theme.clone());
        }
        let _ = self.config.save();
        self.set_popup(format!("Theme: {}", self.config.theme_name), Color::Cyan);
    }

    fn prev_theme(&mut self) {
        let themes = vec!["Default", "Nord", "Dracula", "OneDark", "Gruvbox", "Solarized", "Custom"];
        let current_pos = themes.iter().position(|&t| t == self.config.theme_name).unwrap_or(0);
        let next_pos = if current_pos == 0 { themes.len() - 1 } else { current_pos - 1 };
        self.config.theme_name = themes[next_pos].to_string();
        if self.config.theme_name == "Custom" && self.config.custom_theme.is_none() {
            self.config.custom_theme = Some(self.config.theme.clone());
        }
        let _ = self.config.save();
        self.set_popup(format!("Theme: {}", self.config.theme_name), Color::Cyan);
    }

    fn next_default_tab(&mut self) {
        let tabs = vec!["Search", "Installed", "Updates", "Settings"];
        let current_pos = tabs.iter().position(|&t| t == self.config.settings.default_tab).unwrap_or(0);
        let next_pos = (current_pos + 1) % tabs.len();
        self.config.settings.default_tab = tabs[next_pos].to_string();
        let _ = self.config.save();
        self.set_popup(format!("Default Tab: {}", self.config.settings.default_tab), Color::Cyan);
    }

    fn prev_default_tab(&mut self) {
        let tabs = vec!["Search", "Installed", "Updates", "Settings"];
        let current_pos = tabs.iter().position(|&t| t == self.config.settings.default_tab).unwrap_or(0);
        let next_pos = if current_pos == 0 { tabs.len() - 1 } else { current_pos - 1 };
        self.config.settings.default_tab = tabs[next_pos].to_string();
        let _ = self.config.save();
        self.set_popup(format!("Default Tab: {}", self.config.settings.default_tab), Color::Cyan);
    }

    fn next_border_style(&mut self) {
        let styles = vec!["Plain", "Rounded", "Double", "Thick"];
        let current_pos = styles.iter().position(|&s| s == self.config.settings.border_style).unwrap_or(0);
        let next_pos = (current_pos + 1) % styles.len();
        self.config.settings.border_style = styles[next_pos].to_string();
        let _ = self.config.save();
        self.set_popup(format!("Border Style: {}", self.config.settings.border_style), Color::Cyan);
    }

    fn prev_border_style(&mut self) {
        let styles = vec!["Plain", "Rounded", "Double", "Thick"];
        let current_pos = styles.iter().position(|&s| s == self.config.settings.border_style).unwrap_or(0);
        let next_pos = if current_pos == 0 { styles.len() - 1 } else { current_pos - 1 };
        self.config.settings.border_style = styles[next_pos].to_string();
        let _ = self.config.save();
        self.set_popup(format!("Border Style: {}", self.config.settings.border_style), Color::Cyan);
    }

    fn next_spinner_type(&mut self) {
        let types = vec!["Dots", "Bars", "Pulse", "Classic"];
        let current_pos = types.iter().position(|&t| t == self.config.settings.spinner_type).unwrap_or(0);
        let next_pos = (current_pos + 1) % types.len();
        self.config.settings.spinner_type = types[next_pos].to_string();
        let _ = self.config.save();
        self.set_popup(format!("Spinner: {}", self.config.settings.spinner_type), Color::Cyan);
    }

    fn prev_spinner_type(&mut self) {
        let types = vec!["Dots", "Bars", "Pulse", "Classic"];
        let current_pos = types.iter().position(|&t| t == self.config.settings.spinner_type).unwrap_or(0);
        let next_pos = if current_pos == 0 { types.len() - 1 } else { current_pos - 1 };
        self.config.settings.spinner_type = types[next_pos].to_string();
        let _ = self.config.save();
        self.set_popup(format!("Spinner: {}", self.config.settings.spinner_type), Color::Cyan);
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<Option<String>> {
        loop {
            if let Tab::Search = self.current_tab {
                self.check_and_execute_search();
            }

            if self.loading || matches!(self.details_state, DetailsState::Loading) {
                self.spinner_tick = self.spinner_tick.wrapping_add(1);
            }

            // check for update prompt response
            if self.update_prompt.is_none() {
                if let Ok(Some(update)) = self.update_rx.try_recv() {
                    // A genuinely newer release arrived — clear any stale skip
                    // entry so the config stays tidy.
                    if self.config.settings.skipped_update_version.is_some() {
                        self.config.settings.skipped_update_version = None;
                        let _ = self.config.save();
                    }
                    self.update_prompt = Some(update);
                }
            }

            // check search results
            while let Ok((q, pkgs)) = self.result_rx.try_recv() {
                let is_current_tab_result = match self.current_tab {
                    Tab::Search => q == self.input.trim(),
                    Tab::Installed => q == "__INSTALLED__",
                    Tab::Updates => q == "__UPDATES__",
                    Tab::Settings => false,
                };

                if is_current_tab_result {
                    self.packages = pkgs;

                    self.checked = self
                        .packages
                        .iter()
                        .map(|p| self.selected_names.contains(&p.name))
                        .collect();

                    self.selected = 0;
                    self.last_selected = usize::MAX; // Reset to trigger details for first item
                    self.loading = false;

                    if !self.packages.is_empty() {
                        self.list_state.select(Some(0));
                        self.trigger_details_fetch();
                    } else {
                        self.list_state.select(None);
                        self.details_state = DetailsState::Empty;
                    }

                    self.messages = self
                        .packages
                        .iter()
                        .map(|p| format!("{} {:<15} {}", p.name, p.version, p.description))
                        .collect();
                }
            }

            // check details results
            if let Ok(state) = self.details_rx.try_recv() {
                self.details_state = state;
            }

            if let Some(timer) = self.popup_timer {
                if timer.elapsed() > Duration::from_secs(3) {
                    self.popup_message = None;
                    self.popup_timer = None;
                }
            }

            terminal.draw(|frame| draw_ui(frame, &mut self))?;

            if event::poll(std::time::Duration::from_millis(10))? {
                let ev = event::read()?;
                match ev {
                    Event::Key(key) => {
                        if self.update_prompt.is_some() {
                            if key.kind == KeyEventKind::Press {
                                match key.code {
                                    KeyCode::Left
                                    | KeyCode::Right
                                    | KeyCode::Tab
                                    | KeyCode::Char('h')
                                    | KeyCode::Char('l') => {
                                        self.update_selected_yes = !self.update_selected_yes;
                                    }
                                    KeyCode::Enter => {
                                        let (_, url) = self.update_prompt.take().unwrap();
                                        if self.update_selected_yes {
                                            return Ok(Some(url));
                                        }
                                    }
                                    KeyCode::Esc | KeyCode::Char('q') => {
                                        if let Some((version, _)) = self.update_prompt.take() {
                                            self.config.settings.skipped_update_version = Some(version);
                                            let _ = self.config.save();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            continue;
                        }

                        match self.input_mode {
                            InputMode::Normal if key.kind == KeyEventKind::Press => {
                                let keys = &self.config.keys;
                                
                                // Check custom keybindings
                                let is_key = |code: KeyCode, target: &str| -> bool {
                                    match code {
                                        KeyCode::Char(c) => c.to_string() == target,
                                        KeyCode::Tab => target == "Tab",
                                        KeyCode::BackTab => target == "BackTab",
                                        _ => false,
                                    }
                                };

                                if self.show_help {
                                    if is_key(key.code, &keys.help) || key.code == KeyCode::Esc {
                                        self.show_help = false;
                                        continue;
                                    }
                                    if is_key(key.code, &keys.quit) {
                                        return Ok(None);
                                    }
                                    self.show_help = false;
                                }

                                if is_key(key.code, &keys.quit) {
                                    return Ok(None);
                                } else if is_key(key.code, &keys.help) {
                                    self.show_help = !self.show_help;
                                } else if is_key(key.code, &keys.tab_next) || key.code == KeyCode::Tab {
                                    self.switch_tab();
                                    self.last_selected = usize::MAX;
                                    self.trigger_details_fetch();
                                } else if is_key(key.code, &keys.tab_prev) || key.code == KeyCode::BackTab {
                                    self.switch_tab_previous();
                                    self.last_selected = usize::MAX;
                                    self.trigger_details_fetch();
                                } else if is_key(key.code, &keys.install) {
                                    let _ = self.run_command(terminal);
                                    self.installed_packages = self.manager.get_installed();
                                    if let Tab::Installed = self.current_tab {
                                        self.reset_tab_state();
                                    }
                                } else if is_key(key.code, &keys.remove) {
                                    let _ = self.run_remove_command(terminal);
                                    self.installed_packages = self.manager.get_installed();
                                    if let Tab::Installed = self.current_tab {
                                        self.reset_tab_state();
                                    }
                                } else if is_key(key.code, &keys.system_upgrade) {
                                    let _ = self.manager.system_upgrade(terminal);
                                    self.installed_packages = self.manager.get_installed();
                                    if let Tab::Updates = self.current_tab {
                                        self.reset_tab_state();
                                    }
                                } else if is_key(key.code, &keys.refresh_db) {
                                    let _ = self.manager.refresh_databases(terminal);
                                    if let Tab::Updates = self.current_tab {
                                        self.reset_tab_state();
                                    }
                                } else if is_key(key.code, &keys.toggle_select) {
                                    if self.current_tab == Tab::Settings {
                                        self.handle_settings_toggle();
                                    } else if !self.packages.is_empty() {
                                        let pkg = &self.packages[self.selected];
                                        let name = pkg.name.clone();
                                        let is_checked = !self.checked[self.selected];
                                        self.checked[self.selected] = is_checked;
                                        if is_checked { self.selected_names.insert(name); } else { self.selected_names.remove(&name); }
                                    }
                                } else if is_key(key.code, &keys.search_edit) {
                                    self.show_help = false;
                                    self.input_mode = InputMode::Editing;
                                } else {
                                    match key.code {
                                        KeyCode::Left | KeyCode::Char('h') => {
                                            if self.current_tab == Tab::Settings {
                                                let mgr_count = self.available_managers.len();
                                                if self.settings_index == 6 + mgr_count {
                                                    self.prev_theme();
                                                } else if self.settings_index == 6 + mgr_count + 1 {
                                                    self.prev_border_style();
                                                } else if self.settings_index == 6 + mgr_count + 2 {
                                                    self.prev_spinner_type();
                                                } else if self.settings_index == 4 {
                                                    self.prev_default_tab();
                                                } else if self.settings_index == 1 || self.settings_index == 2 || (self.settings_index >= 6 && self.settings_index < 6 + mgr_count) {
                                                    self.handle_settings_toggle();
                                                }
                                            }
                                        }
                                        KeyCode::Right | KeyCode::Char('l') => {
                                            if self.current_tab == Tab::Settings {
                                                let mgr_count = self.available_managers.len();
                                                if self.settings_index == 6 + mgr_count {
                                                    self.next_theme();
                                                } else if self.settings_index == 6 + mgr_count + 1 {
                                                    self.next_border_style();
                                                } else if self.settings_index == 6 + mgr_count + 2 {
                                                    self.next_spinner_type();
                                                } else if self.settings_index == 4 {
                                                    self.next_default_tab();
                                                } else if self.settings_index == 1 || self.settings_index == 2 || (self.settings_index >= 6 && self.settings_index < 6 + mgr_count) {
                                                    self.handle_settings_toggle();
                                                }
                                            }
                                        }
                                        KeyCode::Up | KeyCode::Char('k') => {
                                            if self.current_tab == Tab::Settings {
                                                if self.settings_index > 0 {
                                                    self.settings_index -= 1;
                                                }
                                            } else if self.selected > 0 {
                                                self.selected -= 1;
                                                self.list_state.select(Some(self.selected));
                                                self.trigger_details_fetch();
                                            }
                                        }
                                        KeyCode::Down | KeyCode::Char('j') => {
                                            if self.current_tab == Tab::Settings {
                                                let max = if self.config.theme_name == "Custom" { 14 } else { 8 };
                                                if self.settings_index < max {
                                                    self.settings_index += 1;
                                                }
                                            } else if self.selected + 1 < self.packages.len() {
                                                self.selected += 1;
                                                self.list_state.select(Some(self.selected));
                                                self.trigger_details_fetch();
                                            }
                                        }
                                        KeyCode::Enter => {
                                            if self.current_tab == Tab::Settings {
                                                self.handle_settings_toggle();
                                            }
                                        }
                                        KeyCode::Home => {
                                            if !self.packages.is_empty() {
                                                self.selected = 0;
                                                self.list_state.select(Some(self.selected));
                                                self.trigger_details_fetch();
                                            }
                                        }
                                        KeyCode::End => {
                                            if !self.packages.is_empty() {
                                                self.selected = self.packages.len() - 1;
                                                self.list_state.select(Some(self.selected));
                                                self.trigger_details_fetch();
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                                KeyCode::Enter => {
                                    if self.current_tab == Tab::Settings {
                                        self.handle_settings_save();
                                    }
                                    self.input_mode = InputMode::Normal;
                                    if self.current_tab == Tab::Search {
                                        self.pending_search = true;
                                        self.last_input_time = Instant::now();
                                    }
                                }
                                KeyCode::Char(c) => self.enter_char(c),
                                KeyCode::Backspace => self.delete_char(),
                                KeyCode::Left => self.move_cursor_left(),
                                KeyCode::Right => self.move_cursor_right(),
                                KeyCode::Esc => self.input_mode = InputMode::Normal,
                                _ => {}
                            }

                            _ => {}
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        self.handle_mouse_event(mouse_event, terminal.size()?.width)?;
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: event::MouseEvent, term_width: u16) -> Result<()> {
        match mouse_event.kind {
            event::MouseEventKind::ScrollDown => {
                if self.current_tab == Tab::Settings {
                    let mgr_count = self.available_managers.len();
                    let max = if self.config.theme_name == "Custom" { 5 + mgr_count + 6 } else { 5 + mgr_count };
                    if self.settings_index < max {
                        self.settings_index += 1;
                    }
                } else {
                    if mouse_event.column > term_width / 2 {
                        self.details_scroll = self.details_scroll.saturating_add(1);
                    } else if self.selected + 1 < self.packages.len() {
                        self.selected += 1;
                        self.list_state.select(Some(self.selected));
                        self.trigger_details_fetch();
                    }
                }
            }
            event::MouseEventKind::ScrollUp => {
                if self.current_tab == Tab::Settings {
                    if self.settings_index > 0 {
                        self.settings_index -= 1;
                    }
                } else {
                    if mouse_event.column > term_width / 2 {
                        self.details_scroll = self.details_scroll.saturating_sub(1);
                    } else if self.selected > 0 {
                        self.selected -= 1;
                        self.list_state.select(Some(self.selected));
                        self.trigger_details_fetch();
                    }
                }
            }
            event::MouseEventKind::Down(event::MouseButton::Left) => {
                // Tab switching
                if mouse_event.row >= 1 && mouse_event.row <= 3 {
                    let col = mouse_event.column.saturating_sub(1);
                    let tab_titles = ["Search", "Installed", "Updates", "Settings"];
                    let mut current_x = 0;
                    for (i, title) in tab_titles.iter().enumerate() {
                        let width = title.len() as u16;
                        if col >= current_x && col < current_x + width {
                            let new_tab = match i {
                                0 => Tab::Search,
                                1 => Tab::Installed,
                                2 => Tab::Updates,
                                3 => Tab::Settings,
                                _ => self.current_tab,
                            };
                            if new_tab != self.current_tab {
                                self.current_tab = new_tab;
                                self.reset_tab_state();
                            }
                            return Ok(());
                        }
                        current_x += width + 3; // 3 for " | " separator in Ratatui Tabs
                    }
                }
                // Settings interaction
                else if self.current_tab == Tab::Settings {
                    let r = mouse_event.row;
                    let mgr_count = self.available_managers.len() as u16;
                    
                    let idx = if r >= 7 && r <= 12 {
                        Some(r - 7)
                    } else if r >= 14 && r < 14 + mgr_count {
                        Some(r - 14 + 6)
                    } else if r == 15 + mgr_count {
                        Some(6 + mgr_count)
                    } else if r == 16 + mgr_count {
                        Some(7 + mgr_count)
                    } else if r == 17 + mgr_count {
                        Some(8 + mgr_count)
                    } else if r >= 19 + mgr_count && r < 25 + mgr_count {
                        Some(r - (19 + mgr_count) + 7 + mgr_count)
                    } else {
                        None
                    };

                    if let Some(i) = idx {
                        self.settings_index = i as usize;
                        if mouse_event.column > 25 {
                            self.handle_settings_toggle();
                        }
                    }
                }
                // List/Details interaction
                else {
                    let is_wide = term_width >= 100;
                    let split_col = if is_wide { term_width / 2 } else { (term_width * 6) / 10 };
                    
                    if mouse_event.column < split_col {
                        // Package List
                        let offset = if self.current_tab == Tab::Search { 7 } else { 4 };
                        if mouse_event.row >= offset {
                            let list_idx = (mouse_event.row - offset) as usize;
                            let state_offset = self.list_state.offset();
                            let real_idx = list_idx + state_offset;
                            if real_idx < self.packages.len() {
                                self.selected = real_idx;
                                self.list_state.select(Some(self.selected));
                                self.trigger_details_fetch();
                                
                                if mouse_event.column < 5 {
                                    let name = self.packages[real_idx].name.clone();
                                    let is_checked = !self.checked[real_idx];
                                    self.checked[real_idx] = is_checked;
                                    if is_checked { self.selected_names.insert(name); } else { self.selected_names.remove(&name); }
                                }
                            }
                        }
                        
                        // Scrollbar click for list
                        if mouse_event.column >= split_col - 2 && mouse_event.column <= split_col - 1 {
                             if mouse_event.row < (term_width / 2) {
                                 if self.selected > 0 { self.selected -= 1; }
                             } else {
                                 if self.selected + 1 < self.packages.len() { self.selected += 1; }
                             }
                             self.list_state.select(Some(self.selected));
                             self.trigger_details_fetch();
                        }
                    } else {
                        // Details Area Scroll
                        if mouse_event.column >= term_width - 2 {
                            if mouse_event.row < (term_width / 4) {
                                self.details_scroll = self.details_scroll.saturating_sub(1);
                            } else {
                                self.details_scroll = self.details_scroll.saturating_add(1);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_toggle(&mut self) {
        let mgr_count = self.available_managers.len();
        match self.settings_index {
            1 => { // Auto Update Check
                self.config.settings.auto_update_check = !self.config.settings.auto_update_check;
                let _ = self.config.save();
                self.set_popup(format!("Auto Update Check: {}", self.config.settings.auto_update_check), Color::Cyan);
            }
            2 => { // Auto Cleanup
                self.config.settings.auto_cleanup = !self.config.settings.auto_cleanup;
                let _ = self.config.save();
                self.set_popup(format!("Auto Cleanup: {}", self.config.settings.auto_cleanup), Color::Cyan);
            }
            i if i >= 6 && i < 6 + mgr_count => {
                let mgr_name = self.available_managers[i - 6].clone();
                self.toggle_manager(&mgr_name);
            }
            i if i == 6 + mgr_count => { 
                self.next_theme(); 
            }
            i if i == 6 + mgr_count + 1 => {
                self.next_border_style();
            }
            i if i == 6 + mgr_count + 2 => {
                self.next_spinner_type();
            }
            _ => {
                // If it's a string/color field, enter editing mode
                self.input = match self.settings_index {
                    0 => self.config.aur_helper.clone(),
                    3 => self.config.settings.search_debounce_ms.to_string(),
                    4 => self.config.settings.default_tab.clone(),
                    5 => self.config.settings.max_search_results.to_string(),
                    i if i >= 7 + mgr_count && i <= 12 + mgr_count && self.config.theme_name == "Custom" => {
                        let theme = self.config.custom_theme.as_ref().unwrap();
                        match i - (7 + mgr_count) {
                            0 => theme.border_color.clone(),
                            1 => theme.highlight_color.clone(),
                            2 => theme.success_color.clone(),
                            3 => theme.error_color.clone(),
                            4 => theme.text_primary.clone(),
                            5 => theme.text_secondary.clone(),
                            _ => String::new(),
                        }
                    }
                    _ => String::new(),
                };
                if !self.input.is_empty() || (self.settings_index >= 7 + mgr_count && self.settings_index <= 12 + mgr_count) {
                    self.input_mode = InputMode::Editing;
                    self.character_index = self.input.chars().count();
                }
            }
        }
    }

    fn handle_settings_save(&mut self) {
        let val = self.input.trim().to_string();
        let mgr_count = self.available_managers.len();
        let mut saved = false;

        match self.settings_index {
            0 => { self.config.aur_helper = val; saved = true; }
            3 => if let Ok(n) = val.parse() { self.config.settings.search_debounce_ms = n; saved = true; },
            4 => { self.config.settings.default_tab = val; saved = true; }
            5 => if let Ok(n) = val.parse() { self.config.settings.max_search_results = n; saved = true; },
            i if i >= 7 + mgr_count && i <= 12 + mgr_count => {
                if let Some(ref mut theme) = self.config.custom_theme {
                    match i - (7 + mgr_count) {
                        0 => theme.border_color = val,
                        1 => theme.highlight_color = val,
                        2 => theme.success_color = val,
                        3 => theme.error_color = val,
                        4 => theme.text_primary = val,
                        5 => theme.text_secondary = val,
                        _ => {}
                    }
                    saved = true;
                }
            }
            _ => {}
        }

        if saved {
            let _ = self.config.save();
            self.set_popup("Settings saved".to_string(), Color::Green);
        } else {
            self.set_popup("Invalid input".to_string(), Color::Red);
        }
    }
}

