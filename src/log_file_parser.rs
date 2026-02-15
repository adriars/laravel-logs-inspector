use std::fs::File;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Seek;
use std::path::PathBuf;
use crate::app::LogEntry;

pub fn parse_log_file(path: PathBuf, last_position: u64) -> LogEntry {
    let mut file_name: String = String::new();
    let mut content: String = String::new();
    let mut current_len: u64 = 0;

    if let Some(name) = path.file_name() {
        file_name = name.to_string_lossy().into_owned();
    }

    // Open the file
    if let Ok(mut file) = File::open(&path) {
        let metadata = file.metadata().unwrap();
        current_len = metadata.len();

        // If the file shrank (e.g., cleared), reset to 0
        let start_pos = if current_len < last_position { 0 } else { last_position };

        if let Ok(_) = file.seek(SeekFrom::Start(start_pos)) {
            let mut new_content = String::new();
            if file.read_to_string(&mut new_content).is_ok() {
                if !new_content.is_empty() {
                    content = new_content;
                }
            }
        }
    }

    return LogEntry { name: file_name, offset: current_len, content: content, new: true };
}