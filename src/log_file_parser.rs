use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use crate::app::LogEntry;

/// Scans the log content line by line, isolates JSON payloads, and beautifies them using serde_json.
fn process_log_content(content: &str) -> String {
    let mut processed = String::with_capacity(content.len() * 2);

    for line in content.lines() {
        // Greedily find the first '{' and the last '}' to capture the potential JSON payload
        if let (Some(first_brace), Some(last_brace)) = (line.find('{'), line.rfind('}')) {
            if first_brace < last_brace {
                let prefix = &line[..first_brace];
                let json_part = &line[first_brace..=last_brace];
                let suffix = &line[last_brace + 1..];

                // Attempt to parse the extracted string as a generic JSON Value
                if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(json_part) {
                    // If valid, format it with pretty indentation
                    if let Ok(beautified) = serde_json::to_string_pretty(&parsed_json) {
                        processed.push_str(prefix);
                        
                        // Drop the JSON payload onto a new line if there is prefix text
                        if !prefix.trim().is_empty() {
                             processed.push('\n');
                        }
                        
                        processed.push_str(&beautified);
                        processed.push_str(suffix);
                        processed.push('\n');
                        continue;
                    }
                }
            }
        }
        
        // If no valid JSON is detected, or parsing fails, append the line as-is
        processed.push_str(line);
        processed.push('\n');
    }

    processed
}

pub fn parse_log_file(path: PathBuf, last_position: u64) -> LogEntry {
    let file_name: String;
    let mut content: String = String::new();
    let mut current_len: u64 = 0;

    file_name = path.to_string_lossy().into_owned();

    // Open the file
    if let Ok(mut file) = File::open(&path) {
        if let Ok(metadata) = file.metadata() {
            current_len = metadata.len();

            // If the file shrank (e.g., cleared), reset to 0
            let start_pos = if current_len < last_position { 0 } else { last_position };

            if file.seek(SeekFrom::Start(start_pos)).is_ok() {
                let mut new_content = String::new();
                if file.read_to_string(&mut new_content).is_ok() {
                    if !new_content.is_empty() {
                        // Process the content to beautify JSON before assigning it
                        content = process_log_content(&new_content);
                    }
                }
            }
        }
    }

    LogEntry { 
        name: file_name, 
        offset: current_len, 
        content, 
        new: true 
    }
}