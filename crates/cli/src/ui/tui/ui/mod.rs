use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use super::app::App;
use super::state::{InputMode, ToastKind};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::style::Style;
use ratatui::layout::Rect;
use std::time::Instant;

pub mod components;
pub mod utils;

use components::{input, overlay, status, transcript, command_palette};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    transcript::draw(f, app, chunks[0]);
    input::draw(f, app, chunks[1]);
    status::draw(f, app, chunks[2]);

    if app.state.mode == InputMode::CommandPalette {
        command_palette::draw(f, app);
    }

    if app.state.mode == InputMode::Overlay {
        overlay::draw(f, app);
    }

    if let Some(toast) = &app.state.toast {
         if Instant::now() <= toast.expires_at {
             let width = (toast.message.len() as u16 + 4).min(f.size().width.saturating_sub(4));
             let height = 3;
             let area = Rect::new(
                 f.size().width.saturating_sub(width + 2),
                 2,
                 width,
                 height
             );
             f.render_widget(Clear, area);
             
             let color = match toast.kind {
                 ToastKind::Success => app.theme.success,
                 ToastKind::Info => app.theme.user,
                 ToastKind::Error => app.theme.error,
             };
             
             let block = Block::default()
                 .borders(Borders::ALL)
                 .border_style(Style::default().fg(color));
             
             let text = Paragraph::new(toast.message.clone())
                 .block(block)
                 .alignment(ratatui::layout::Alignment::Center)
                 .style(Style::default().fg(color));
             
             f.render_widget(text, area);
         }
    }
}
