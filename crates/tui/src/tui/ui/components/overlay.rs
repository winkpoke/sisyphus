use crate::tui::app::App;
use crate::tui::ui::utils::centered_rect;
use ratatui::{
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.size());
    f.render_widget(Clear, area);

    let style = if app.state.overlay.is_error {
        Style::default().fg(app.theme.error)
    } else {
        Style::default().fg(app.theme.border)
    };

    let p = Paragraph::new(app.state.overlay.content.clone())
        .block(
            Block::default()
                .title(app.state.overlay.title.clone())
                .borders(Borders::ALL)
                .border_style(style),
        )
        .scroll((app.state.overlay.scroll, 0))
        .wrap(Wrap { trim: true });

    f.render_widget(p, area);
}
