use crate::tui::app::App;
use crate::tui::state::AppStatus;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Left: Status
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let status_text = match app.state.status {
        AppStatus::Connected => Span::styled("● Connected", Style::default().fg(app.theme.success)),
        AppStatus::Disconnected => {
            Span::styled("○ Disconnected", Style::default().fg(app.theme.error))
        }
        AppStatus::Processing => {
            let frame = app.state.spinner_frame % spinner_chars.len();
            Span::styled(
                format!("{} Processing", spinner_chars[frame]),
                Style::default().fg(app.theme.highlight),
            )
        }
    };
    let status_widget = Paragraph::new(Line::from(status_text));
    f.render_widget(status_widget, status_chunks[0]);

    // Center: Model & Token Usage
    let model_info = Paragraph::new(format!(
        "{} | {}",
        app.state.active_model, app.state.token_usage
    ))
    .alignment(ratatui::layout::Alignment::Center)
    .style(Style::default().fg(app.theme.system));
    f.render_widget(model_info, status_chunks[1]);

    // Right: Session ID
    let session_info = Paragraph::new(format!("Session: {}", app.state.session_id))
        .alignment(ratatui::layout::Alignment::Right)
        .style(Style::default().fg(app.theme.user));
    f.render_widget(session_info, status_chunks[2]);
}
