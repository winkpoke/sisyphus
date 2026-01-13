use crate::tui::app::App;
use crate::tui::ui::utils::centered_rect;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    if let Some(state) = &app.state.agent_selection {
        let area = centered_rect(60, 40, f.size());
        f.render_widget(Clear, area);

        let items: Vec<ListItem> = state
            .agents
            .iter()
            .map(|agent| {
                let name = Span::styled(
                    format!("{} ({})", agent.name, agent.model),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                );
                let desc = Span::styled(
                    format!(" - {}", agent.description),
                    Style::default().fg(Color::Gray),
                );
                let id = Span::styled(
                    format!(" [{}]", agent.id),
                    Style::default().fg(Color::DarkGray),
                );

                ListItem::new(Line::from(vec![name, id, desc]))
            })
            .collect();

        let title = "Select Agent";
        let list = List::new(items)
            .block(Block::default().title(title).borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .fg(app.theme.highlight)
                    .add_modifier(Modifier::REVERSED),
            )
            .highlight_symbol("> ");

        let mut list_state = ListState::default();
        list_state.select(Some(state.selected_index));

        f.render_stateful_widget(list, area, &mut list_state);
    }
}
