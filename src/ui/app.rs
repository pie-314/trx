use crate::ui::{draw::draw_ui, input::InputMode};
use color_eyre::Result;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    widgets::ListState,
};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use crate::pacman::{self, Package};

pub struct App {
    pub input: String,
    pub character_index: usize,
    pub input_mode: InputMode,
    pub packages: Vec<Package>,
    pub selected: usize,
    pub list_state: ListState,
    pub messages: Vec<String>,
    pub loading: bool,
    result_tx: Sender<Vec<Package>>,
    result_rx: Receiver<Vec<Package>>,
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
            selected: 0,
            list_state,
            loading: false,
            result_tx,
            result_rx,
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

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
        self.submit_message();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let left = self.character_index - 1;
            let before = self.input.chars().take(left);
            let after = self.input.chars().skip(self.character_index);
            self.input = before.chain(after).collect();
            self.move_cursor_left();
            self.submit_message();
        }
    }

    fn clamp_cursor(&self, new_pos: usize) -> usize {
        new_pos.clamp(0, self.input.chars().count())
    }

    fn submit_message(&mut self) {
        let query = self.input.trim().to_string();
        if query.is_empty() {
            return;
        }

        self.loading = true;
        let tx = self.result_tx.clone();

        thread::spawn(move || {
            let pkgs = pacman::search_vec(&query);
            let _ = tx.send(pkgs);
        });
    }

    fn install_package(&mut self) {
        if let Some(pkg) = self.packages.get(self.selected).cloned() {
            thread::spawn(move || {
                pacman::package_installer(&pkg.name);
            });
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            if let Ok(pkgs) = self.result_rx.try_recv() {
                self.packages = pkgs;
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
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('i') => self.install_package(),
                            KeyCode::Up => {
                                if !self.packages.is_empty() && self.selected > 0 {
                                    self.selected -= 1;
                                    self.list_state.select(Some(self.selected));
                                }
                            }
                            KeyCode::Down => {
                                if !self.packages.is_empty()
                                    && self.selected + 1 < self.packages.len()
                                {
                                    self.selected += 1;
                                    self.list_state.select(Some(self.selected));
                                }
                            }
                            _ => {}
                        },
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => {
                                self.submit_message();
                                self.input_mode = InputMode::Normal;
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

