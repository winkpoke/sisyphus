use crate::tui::app::App;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let commands = &app.state.command_palette.filtered_commands;
    if commands.is_empty() {
        return;
    }

    let height = (commands.len() as u16).min(10) + 2; // +2 for borders
    let width = 40;

    // Position at bottom left, above input
    // Input is 1 line high, so we subtract height + 1
    let area = Rect::new(0, f.size().height.saturating_sub(height + 1), width, height);

    f.render_widget(Clear, area);

    let items: Vec<ListItem> = commands
        .iter()
        .map(|cmd| ListItem::new(Line::from(cmd.as_str())))
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border))
        .title(format!("Commands ({})", app.state.command_palette.input));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.state.command_palette.selected_index));

    f.render_stateful_widget(list, area, &mut state);
}
