use crate::tui::app::App;
use crate::tui::state::AppStatus;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(area);

    // Input Area
    let prompt_style = if !app.state.input_buffer.is_empty() {
        Style::default().fg(app.theme.highlight)
    } else {
        Style::default().fg(app.theme.system)
    };

    let prompt_span = Span::styled("> ", prompt_style);
    let input_span = Span::raw(app.state.input_buffer.clone());
    
    let input_line = Line::from(vec![prompt_span, input_span]);
    let input_para = Paragraph::new(input_line);
    f.render_widget(input_para, chunks[0]);

    // Status Area
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let status_span = match app.state.status {
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

    let token_span = Span::styled(
        format!(" [{}]", app.state.token_usage),
        Style::default().fg(app.theme.system),
    );
    
    // Optional: Session ID if needed, but ContextBar might be enough.
    // Proposal says "Session ID moved to status footer" in "Impact".
    let session_span = Span::styled(
        format!(" [{}]", app.state.session_id),
        Style::default().fg(app.theme.user),
    );

    let status_line = Line::from(vec![
        status_span,
        token_span,
        session_span,
    ]);
    
    let status_para = Paragraph::new(status_line).alignment(Alignment::Right);
    f.render_widget(status_para, chunks[1]);
}
