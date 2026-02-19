
use ratatui::{crossterm::event::Event, widgets::ListState};

pub enum AppEvent {
    FileCreated(String),
    FileUpdated(String),
    TerminalEvent(Event),
}

pub struct LogEntry {
    pub name: String,
    pub offset: u64,
    pub content: String,
    pub new: bool
}

pub struct App {
    pub log_entries: Vec<LogEntry>,
    pub log_entries_list_state: ListState,
}

impl App {
    pub fn new() -> App {
        App {
            log_entries: Vec::new(),
            log_entries_list_state: ListState::default(),
        }
    }

    pub fn select_next_log_entry(&mut self) {
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
        self.log_entries_list_state.select_first();
    }

    pub fn select_last_log_entry(&mut self) {
        self.log_entries_list_state.select_last();
    }

    pub fn make_current_log_entries_old(&mut self) {
        for log_entry in self.log_entries.iter_mut() {
            log_entry.new = false;
            log_entry.content.clear();
        }
    }
}
