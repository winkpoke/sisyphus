use super::app::App;
use super::state::{InputMode, ToastKind};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use std::time::Instant;

pub mod components;
pub mod utils;

use components::{agent_selection, command_palette, context_bar, input, overlay, transcript};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();
    
    // Critical size check: < 40x10
    if size.width < 40 || size.height < 10 {
        let warning_block = Paragraph::new("Terminal too small.\nPlease resize to at least 40x10.")
            .style(Style::default().fg(app.theme.error))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(warning_block, size);
        return;
    }

    // Degraded mode check: < 80x24
    let degraded_mode = size.width < 80 || size.height < 24;

    let chunks = if degraded_mode {
        // Degraded layout: No ContextBar, just Transcript and Input
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Min(1), // Transcript
                    Constraint::Length(1), // Input
                ]
                .as_ref(),
            )
            .split(size)
    } else {
        // Full layout
        Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(1), // ContextBar
                    Constraint::Min(1),    // Transcript
                    Constraint::Length(1), // Input
                ]
                .as_ref(),
            )
            .split(size)
    };

    if degraded_mode {
        transcript::draw(f, app, chunks[0]);
        input::draw(f, app, chunks[1]);
        // No ContextBar
    } else {
        context_bar::draw(f, app, chunks[0]);
        transcript::draw(f, app, chunks[1]);
        input::draw(f, app, chunks[2]);
    }

    if app.state.mode == InputMode::CommandPalette {
        command_palette::draw(f, app);
    }

    if app.state.mode == InputMode::AgentSelection {
        agent_selection::draw(f, app);
    }

    if app.state.mode == InputMode::Overlay {
        overlay::draw(f, app);
    }

    if let Some(toast) = &app.state.toast {
        if Instant::now() <= toast.expires_at {
            let width = (toast.message.len() as u16 + 4).min(f.size().width.saturating_sub(4));
            let height = 3;
            let area = Rect::new(f.size().width.saturating_sub(width + 2), 2, width, height);
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
