use crate::tui::app::App;
use crate::tui::ui::utils::centered_rect;
use ratatui::{
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.size());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app
        .state
        .command_palette
        .filtered_commands
        .iter()
        .map(|c| ListItem::new(Line::from(c.as_str())))
        .collect();

    let title = format!(
        "Command Palette (Filter: {})",
        app.state.command_palette.input
    );
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(app.theme.highlight)
                .add_modifier(Modifier::REVERSED),
        )
        .highlight_symbol("> ");

    let mut state = ListState::default();
    state.select(Some(app.state.command_palette.selected_index));

    f.render_stateful_widget(list, area, &mut state);
}
