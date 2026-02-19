mod app;
mod log_file_parser;
mod log_file_watcher;
mod ui;

use std::{env, error::Error, io, path, sync::mpsc, thread};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
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
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
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
    let folder_path: String;

    if args.len() > 1 {
        folder_path = args[1].clone();
    } else {
        folder_path = "./".to_string();
    }

    // The Central Channel
    let (tx, rx) = mpsc::channel();

    // create file watcher thread
    let log_file_watcher = LogFileWatcher::new(folder_path, tx.clone());
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

                let new_entry = log_file_parser::parse_log_file(name.into(), 0);

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

                let content_length = match existing_index {
                    Some(index) => app.log_entries[index].content.len(),
                    None => 0,
                };
                
                // 2. Parse the new data using the found offset
                let new_entry = log_file_parser::parse_log_file(name.into(), offset);
                
                // 3. Update the array
                if let Some(index) = existing_index {
                    // Option A: Replace the old entry at the same position
                    app.log_entries[index] = new_entry;
                } else {
                    // Option B: It's truly new, so push it
                    app.log_entries.push(new_entry);
                }
            }
            AppEvent::TerminalEvent(Event::Key(key_event)) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(false);
                        }
                        KeyCode::Down => {
                            app.select_next_log_entry();
                        }
                        KeyCode::Up => {
                            app.select_previous_log_entry();
                        }
                        KeyCode::Home => {
                            app.select_first_log_entry();
                        }
                        KeyCode::End => {
                            app.select_last_log_entry();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
