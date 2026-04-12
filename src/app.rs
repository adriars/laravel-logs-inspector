use std::path::PathBuf;

use ratatui::{crossterm::event::Event, widgets::ListState};

use crate::app;

pub enum AppEvent {
    FileCreated(PathBuf),
    FileUpdated(PathBuf),
    TerminalEvent(Event),
}

pub struct LogEntry {
    pub name: String,
    pub offset: u64,
    pub content: String,
    pub new: bool,
}

pub struct App {
    pub log_entries: Vec<LogEntry>,
    pub log_entries_list_state: ListState,
    pub paragraph_scroll: (u16, u16),
    pub debug_mode: bool,
}

impl App {
    pub fn new() -> App {
        App {
            log_entries: Vec::new(),
            log_entries_list_state: ListState::default(),
            paragraph_scroll: (0, 0),
            debug_mode: false,
        }
    }

    pub fn select_next_log_entry(&mut self) {
        self.reset_paragraph_scroll();

        if !self.log_entries.is_empty() {
            let i = match self.log_entries_list_state.selected() {
                Some(i) => {
                    if i >= self.log_entries.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.log_entries_list_state.select(Some(i));
        }
    }

    pub fn select_previous_log_entry(&mut self) {
        self.reset_paragraph_scroll();

        if !self.log_entries.is_empty() {
            let i = match self.log_entries_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.log_entries.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.log_entries_list_state.select(Some(i));
        }
    }

    pub fn select_first_log_entry(&mut self) {
        self.reset_paragraph_scroll();

        self.log_entries_list_state.select_first();
    }

    pub fn select_last_log_entry(&mut self) {
        self.reset_paragraph_scroll();

        self.log_entries_list_state.select_last();
    }

    pub fn make_current_log_entries_old(&mut self) {
        self.reset_paragraph_scroll();

        for log_entry in self.log_entries.iter_mut() {
            log_entry.new = false;
        }
    }

    pub fn scroll_down_paragraph(&mut self) {
        self.paragraph_scroll = (self.paragraph_scroll.0 + 1, self.paragraph_scroll.1);
    }

    pub fn scroll_up_paragraph(&mut self) {
        if self.paragraph_scroll.0 > 0 {
            self.paragraph_scroll = (self.paragraph_scroll.0 - 1, self.paragraph_scroll.1);
        }
    }

    pub fn toggle_debug_mode(&mut self) {
        self.debug_mode = !self.debug_mode;
    }

    fn reset_paragraph_scroll(&mut self) {
        self.paragraph_scroll = (0, 0);
    }
}
