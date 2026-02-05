use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Padding, Paragraph, Wrap},
};

use crate::app::{App, CurrentScreen};

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(12),
            Constraint::Percentage(12),
            Constraint::Fill(1),
        ])
        .split(frame.area());

    // Create dynamic styles
    let log_files_block_style: Style;
    let log_entries_block_style: Style;
    let log_content_block_style: Style;

    match app.current_screen {
        CurrentScreen::NavigatingLogFiles => {
            log_files_block_style = Style::default().fg(Color::Magenta);
            log_entries_block_style = Style::default();
            log_content_block_style = Style::default();
        }
        CurrentScreen::NavigatingLogEntries => {
            log_files_block_style = Style::default();
            log_entries_block_style = Style::default().fg(Color::Magenta);
            log_content_block_style = Style::default();
        }
        CurrentScreen::NavigatingLogContent => {
            log_files_block_style = Style::default();
            log_entries_block_style = Style::default();
            log_content_block_style = Style::default().fg(Color::Magenta);
        }
    }

    let mut log_files_list_items = Vec::<ListItem>::new();
    let mut log_entries_list_items = Vec::<ListItem>::new();

    for file in &app.log_files {
        log_files_list_items.push(ListItem::new(Line::from(Span::styled(
            format!("{}", file.name),
            Style::default().fg(Color::White),
        ))));
    }

    if let Some(selected_file_index) = app.log_files_list_state.selected() {
        if let Some(selected_file) = app.log_files.get(selected_file_index) {
            for log_entry in &selected_file.logs {
                log_entries_list_items.push(ListItem::new(Line::from(Span::styled(
                    format!("{}", log_entry),
                    Style::default().fg(Color::White),
                ))));
            }

            if let Some(selected_entry_index) = app.log_entries_list_state.selected() {
                if let Some(selected_content) = selected_file.content.get(selected_entry_index) {
                    let paragraph = Paragraph::new(selected_content.to_string())
                        .block(
                            Block::bordered()
                                .padding(Padding::new(1, 1, 1, 1))
                                .border_style(log_content_block_style),
                        )
                        .alignment(Alignment::Center)
                        .wrap(Wrap { trim: true });

                    frame.render_widget(paragraph, chunks[2]);
                }
            }
        }
    }

    let log_files_list = List::new(log_files_list_items)
        .block(Block::bordered().style(log_files_block_style))
        .highlight_style(Style::default().bg(Color::Magenta).fg(Color::White));

    let log_entries_list = List::new(log_entries_list_items)
        .block(Block::bordered().style(log_entries_block_style))
        .highlight_style(Style::default().bg(Color::Magenta).fg(Color::White));

    frame.render_stateful_widget(log_files_list, chunks[0], &mut app.log_files_list_state);
    frame.render_stateful_widget(log_entries_list, chunks[1], &mut app.log_entries_list_state);
}
