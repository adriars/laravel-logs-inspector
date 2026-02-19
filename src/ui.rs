use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Padding, Paragraph, Wrap},
};

use crate::app::App;

pub fn ui(frame: &mut Frame, app: &mut App) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(12),
            Constraint::Fill(1),
            Constraint::Percentage(12),
        ])
        .split(frame.area());

    let mut log_entries_list_items = Vec::<ListItem>::new();

    for log_entry in &app.log_entries {
        if log_entry.new {
            log_entries_list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", log_entry.name),
                Style::default().fg(Color::LightMagenta),
            ))));
        } else {
            log_entries_list_items.push(ListItem::new(Line::from(Span::styled(
                format!("{}", log_entry.name),
                Style::default().fg(Color::White),
            ))));
        }
    }

    let log_entries_list = List::new(log_entries_list_items)
        .block(Block::bordered().style(Style::default().fg(Color::Magenta)))
        .highlight_style(Style::default().bg(Color::Magenta).fg(Color::White));

    frame.render_stateful_widget(log_entries_list, chunks[0], &mut app.log_entries_list_state);

    if let Some(log_entry_selected_index) = app.log_entries_list_state.selected() {
        let paragraph = Paragraph::new(
            app.log_entries[log_entry_selected_index]
                .content
                .to_string(),
        )
        .block(
            Block::bordered()
                .padding(Padding::new(1, 1, 1, 1))
                .border_style(Style::default().fg(Color::LightMagenta)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, chunks[1]);

        let debug_info = Paragraph::new(format!(
            "name: {} offset: {} content: {} content_length: {} new: {}",
            app.log_entries[log_entry_selected_index].name,
            app.log_entries[log_entry_selected_index].offset,
            app.log_entries[log_entry_selected_index].content,
            app.log_entries[log_entry_selected_index].content.len(),
            app.log_entries[log_entry_selected_index].new
        ))
        .block(
            Block::bordered()
                .padding(Padding::new(1, 1, 1, 1))
                .border_style(Style::default().fg(Color::LightMagenta)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        frame.render_widget(debug_info, chunks[2]);
    }
}
