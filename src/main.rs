mod app;
mod log_file_parser;
mod log_file_watcher;
mod ui;

use std::{env, error::Error, io, path::{self, PathBuf}, sync::mpsc, thread};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use crate::{
    app::{App, AppEvent, LogEntry},
    log_file_watcher::LogFileWatcher,
    ui::ui,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    // Get the parameters passed to the terminal
    let args: Vec<String> = env::args().collect();
    let folder_path: PathBuf;

    if args.len() > 1 {
        // folder_path = args[1].clone();
        let terminal_path_argument = args[1].clone();
        folder_path = std::fs::canonicalize(terminal_path_argument.clone()).unwrap_or_else(|_| std::path::PathBuf::from(terminal_path_argument));
    } else {
        folder_path = std::fs::canonicalize("./").unwrap_or_else(|_| std::path::PathBuf::from("./"));
    }

    app.folder_path = folder_path.clone();

    // The Central Channel
    let (tx, rx) = mpsc::channel();

    // create file watcher thread
    let log_file_watcher = LogFileWatcher::new(folder_path.clone(), tx.clone());
    log_file_watcher.start();

    // create a terminal input watcher thread
    let input_tx = tx.clone();
    thread::spawn(move || {
        loop {
            let _ = input_tx.send(AppEvent::TerminalEvent(event::read().unwrap()));
        }
    });

    loop {
        // Draw the TUI
        terminal.draw(|f| ui(f, app))?;

        // This blocks until *any* event arrives, using 0% CPU while waiting
        match rx.recv().unwrap() {
            AppEvent::FileCreated(name) => {
                app.make_current_log_entries_old();

                let new_entry = log_file_parser::parse_log_file(name, 0);

                app.log_entries.push(new_entry);
            }
            AppEvent::FileUpdated(name) => {
                app.make_current_log_entries_old();
                
                // 1. Find the index of the existing entry if it exists
                let existing_index = app.log_entries.iter().position(|le| le.name == name);

                let offset = match existing_index {
                    Some(index) => app.log_entries[index].offset,
                    None => 0,
                };
                
                // 2. Parse the new data using the found offset
                let new_entry = log_file_parser::parse_log_file(name, offset);
                
                // 3. Update the array
                app.log_entries.insert(0,new_entry);
            }
            AppEvent::TerminalEvent(Event::Key(key_event)) => {
                if key_event.kind == KeyEventKind::Press {
                    match (key_event.code, key_event.modifiers) {
                        // No modifier key
                        (KeyCode::Char('q') | KeyCode::Esc, KeyModifiers::NONE) => {
                            return Ok(false);
                        }
                        (KeyCode::Down, KeyModifiers::NONE) => {
                            app.select_next_log_entry();
                        }
                        (KeyCode::Up, KeyModifiers::NONE) => {
                            app.select_previous_log_entry();
                        }
                        (KeyCode::Home, KeyModifiers::NONE) => {
                            app.select_first_log_entry();
                        }
                        (KeyCode::End, KeyModifiers::NONE) => {
                            app.select_last_log_entry();
                        }
                        (KeyCode::Char('d'), KeyModifiers::NONE) => {
                            app.toggle_debug_mode();
                        }
                        // Shift key modifier
                        (KeyCode::Down, KeyModifiers::SHIFT) => {
                            app.scroll_down_paragraph();
                        }
                        (KeyCode::Up, KeyModifiers::SHIFT) => {
                            app.scroll_up_paragraph();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
