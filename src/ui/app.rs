use crate::ui::{draw::draw_ui, input::InputMode};
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::ListState,
};
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::managers::{self, Package};

pub struct App {
    pub input: String,
    pub character_index: usize,
    pub input_mode: InputMode,
    pub packages: Vec<Package>,
    pub checked: Vec<bool>,
    pub selected_names: HashSet<String>,
    pub selected: usize,
    pub list_state: ListState,
    pub messages: Vec<String>,
    pub loading: bool,
    pub details: Option<std::collections::HashMap<String, String>>,
    pub last_selected: usize,
    result_tx: Sender<Vec<Package>>,
    result_rx: Receiver<Vec<Package>>,
    last_input_time: Instant,
    pending_search: bool,
    last_search_query: String,
}

impl App {
    pub fn new(result_tx: Sender<Vec<Package>>, result_rx: Receiver<Vec<Package>>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(None);

        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
            packages: Vec::new(),
            checked: Vec::new(),
            selected_names: HashSet::new(),
            selected: 0,
            list_state,
            loading: false,
            details: None,
            last_selected: usize::MAX,
            result_tx,
            result_rx,
            last_input_time: Instant::now(),
            pending_search: false,
            last_search_query: String::new(),
        }
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
        const DEBOUNCE_MS: u64 = 100;

        if self.pending_search
            && self.last_input_time.elapsed() >= Duration::from_millis(DEBOUNCE_MS)
        {
            let query = self.input.trim().to_string();

            if !query.is_empty() && query != self.last_search_query {
                self.last_search_query = query.clone();
                self.pending_search = false;
                self.loading = true;

                let tx = self.result_tx.clone();

                thread::spawn(move || {
                    let pac_handle = thread::spawn({
                        let q = query.clone();
                        move || managers::search_pacman(&q)
                    });

                    let aur_handle = thread::spawn({
                        let q = query.clone();
                        move || managers::search_aur(&q)
                    });

                    let mut all = pac_handle.join().unwrap_or_default();
                    all.extend(aur_handle.join().unwrap_or_default());

                    all.sort_by(|a, b| {
                        b.score
                            .partial_cmp(&a.score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });

                    all.truncate(50);

                    let _ = tx.send(all);
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

        let mut pacman_pkgs = HashSet::new();
        let mut aur_pkgs = HashSet::new();

        for name in &self.selected_names {
            if let Some(pkg) = self.packages.iter().find(|p| p.name == *name) {
                match pkg.provider.as_str() {
                    "pacman" => {
                        pacman_pkgs.insert(name.clone());
                    }
                    "aur" => {
                        aur_pkgs.insert(name.clone());
                    }
                    _ => {}
                }
            }
        }

        if !pacman_pkgs.is_empty() {
            managers::pacman_installation(terminal, &pacman_pkgs)?;
        }

        if !aur_pkgs.is_empty() {
            managers::aur_installation(terminal, &aur_pkgs)?;
        }

        Ok(())
    }

    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            self.check_and_execute_search();

            if let Ok(pkgs) = self.result_rx.try_recv() {
                self.packages = pkgs;

                self.checked = self
                    .packages
                    .iter()
                    .map(|p| self.selected_names.contains(&p.name))
                    .collect();

                self.selected = 0;
                self.loading = false;

                if !self.packages.is_empty() {
                    self.list_state.select(Some(0));
                } else {
                    self.list_state.select(None);
                }

                self.messages = self
                    .packages
                    .iter()
                    .map(|p| format!("{} {:<15} {}", p.name, p.version, p.description))
                    .collect();
            }

            terminal.draw(|frame| draw_ui(frame, &mut self))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match self.input_mode {
                        InputMode::Normal if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Char('i') => {
                                let _ = self.run_command(terminal);
                            }
                            KeyCode::Char(' ') => {
                                if !self.packages.is_empty() {
                                    let pkg = &self.packages[self.selected];
                                    let name = pkg.name.clone();

                                    let is_checked = !self.checked[self.selected];
                                    self.checked[self.selected] = is_checked;

                                    if is_checked {
                                        self.selected_names.insert(name);
                                    } else {
                                        self.selected_names.remove(&name);
                                    }
                                }
                            }
                            KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                            KeyCode::Char('q') => return Ok(()),

                            KeyCode::Up | KeyCode::Char('k') => {
                                if self.selected > 0 {
                                    self.selected -= 1;
                                    self.list_state.select(Some(self.selected));
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if self.selected + 1 < self.packages.len() {
                                    self.selected += 1;
                                    self.list_state.select(Some(self.selected));
                                }
                            }
                            _ => {}
                        },

                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => {
                                self.input_mode = InputMode::Normal;
                                self.pending_search = true;
                                self.last_input_time = Instant::now();
                            }
                            KeyCode::Char(c) => self.enter_char(c),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        },

                        _ => {}
                    }
                }
            }
        }
    }
}
