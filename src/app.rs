use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use ratatui::widgets::ListState;
use regex::Regex;

pub enum CurrentScreen {
    NavigatingLogFiles,
    NavigatingLogEntries,
    NavigatingLogContent
}

pub struct LogFile {
    pub name: String,
    pub logs: Vec<String>,
    pub content: Vec<String>
}

pub struct App {
    pub log_files: Vec<LogFile>,
    pub current_screen: CurrentScreen,
    pub log_files_list_state: ListState,
    pub log_entries_list_state: ListState
}

impl App {
    pub fn new() -> App {
        App {
            log_files: Vec::new(),
            current_screen: CurrentScreen::NavigatingLogFiles,
            log_files_list_state: ListState::default().with_selected(Some(0)),
            log_entries_list_state: ListState::default().with_selected(Some(0))
        }
    }

    pub fn select_next_log_file(&mut self) {
        let i = match self.log_files_list_state.selected() {
            Some(i) => if i >= self.log_files.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.log_files_list_state.select(Some(i));
    }

    pub fn select_previous_log_file(&mut self) {
        let i = match self.log_files_list_state.selected() {
            Some(i) => if i == 0 { self.log_files.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.log_files_list_state.select(Some(i));
    }

    pub fn select_first_log_file(&mut self) {
        self.log_files_list_state.select_first();
    }

    pub fn select_last_log_file(&mut self) {
        self.log_files_list_state.select_last();
    }

    pub fn select_next_log_entry(&mut self) {

        let mut log_entries_list_size = 0;

        if let Some(index) = self.log_files_list_state.selected() {
            if let Some(selected_file) = self.log_files.get(index) {
                log_entries_list_size = selected_file.content.len();
            }
        }

        let i = match self.log_entries_list_state.selected() {
            Some(i) => if i >= log_entries_list_size - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.log_entries_list_state.select(Some(i));
    }

    pub fn select_previous_log_entry(&mut self) {

        let mut log_entries_list_size = 0;

        if let Some(index) = self.log_files_list_state.selected() {
            if let Some(selected_file) = self.log_files.get(index) {
                log_entries_list_size = selected_file.content.len();
            }
        }

        let i = match self.log_entries_list_state.selected() {
            Some(i) => if i == 0 { log_entries_list_size - 1 } else { i - 1 },
            None => 0,
        };
        self.log_entries_list_state.select(Some(i));
    }

    pub fn select_first_log_entry(&mut self) {
        self.log_entries_list_state.select_first();
    }

    pub fn select_last_log_entry(&mut self) {
        self.log_entries_list_state.select_last();
    }

    pub fn process_log_files(&mut self, dir_path: &str) -> io::Result<()> {
    // 1. Collect and filter the paths
    let valid_paths: Vec<_> = std::fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.to_string_lossy().contains("logs"))
        .collect();

    self.log_files.clear();

    // Compile the regex once. 
    // Matches: [YYYY-MM-DD HH:MM:SS]
    let re = Regex::new(r"\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\]").expect("Invalid Regex");

    // 2. Process the results
    for path in valid_paths {
        let mut file_name: String = String::new();
        if let Some(name) = path.file_name() {
            file_name = name.to_string_lossy().into_owned();
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        // Vectors to hold the parsed data for this specific file
        let mut file_logs: Vec<String> = Vec::new();
        let mut file_content: Vec<String> = Vec::new();

        // Buffer to accumulate lines for the current log entry
        let mut current_content_buffer = String::new();
        let mut inside_log_block = false;

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(e) if e.kind() == std::io::ErrorKind::InvalidData => continue,
                Err(e) => return Err(e),
            };

            // Check if this line contains a new timestamp
            if let Some(mat) = re.find(&line) {
                // If we were already building a log, save the PREVIOUS log's content now
                if inside_log_block {
                    file_content.push(current_content_buffer.trim().to_string());
                    current_content_buffer.clear();
                }

                // 1. Push the new timestamp (the log identifier)
                file_logs.push(mat.as_str().to_string());

                // 2. Start capturing content for this new log
                // (We take everything after the timestamp on this line)
                let content_part = line[mat.end()..].to_string();
                current_content_buffer.push_str(&content_part);
                
                inside_log_block = true;
            } else if inside_log_block {
                // If we are currently inside a block, just append the line
                // Add a newline to preserve formatting if the buffer isn't empty
                if !current_content_buffer.is_empty() {
                    current_content_buffer.push('\n');
                }
                current_content_buffer.push_str(&line);
            }
        }

        // Don't forget to push the content of the very last log entry
        if inside_log_block {
            file_content.push(current_content_buffer.trim().to_string());
        }

        // Only push the LogFile if we actually found logs
        if !file_logs.is_empty() {
            self.log_files.push(LogFile {
                name: file_name,
                logs: file_logs,
                content: file_content,
            });
        }
    }

    Ok(())
}
}
