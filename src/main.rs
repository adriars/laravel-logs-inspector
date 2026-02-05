use std::{error::Error, io, time::Duration};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

mod app;
mod ui;
use crate::{
    app::{App, CurrentScreen},
    ui::ui,
};

// The duration of a frame in seconds
// the application refreshes its UI every this amount of seconds
// and/or every time the user sends any input
const FRAME_TIME: u64 = 1;

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
    loop {
        app.process_log_files("./")?;

        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_secs(FRAME_TIME))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    return Ok(false);
                }
                match app.current_screen {
                    CurrentScreen::NavigatingLogFiles => match key.code {
                        KeyCode::Down => {
                            app.select_next_log_file();
                        },
                        KeyCode::Up => {
                            app.select_previous_log_file();
                        },
                        KeyCode::Home => {
                            app.select_first_log_file();
                        },
                        KeyCode::End => {
                            app.select_last_log_file();
                        }
                        KeyCode::Right => {
                            app.current_screen = CurrentScreen::NavigatingLogEntries;
                        }
                        _ => {}
                    },
                    CurrentScreen::NavigatingLogEntries if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Down => {
                            app.select_next_log_entry();
                        },
                        KeyCode::Up => {
                            app.select_previous_log_entry();
                        },
                        KeyCode::Home => {
                            app.select_first_log_entry();
                        },
                        KeyCode::End => {
                            app.select_last_log_entry();
                        },
                        KeyCode::Left => {
                            app.current_screen = CurrentScreen::NavigatingLogFiles;
                        },
                        KeyCode::Right => {
                            app.current_screen = CurrentScreen::NavigatingLogContent;
                        }
                        _ => {}
                    },
                    CurrentScreen::NavigatingLogContent => match key.code {
                        KeyCode::Left => {
                            app.current_screen = CurrentScreen::NavigatingLogEntries;
                        },
                        _=> {}
                    }
                    _ => {
                    }
                }
            }
        }
    }
}
