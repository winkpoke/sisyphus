use crate::tui::app::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(app.theme.context_bar_fg))
        .style(
            Style::default()
                .bg(app.theme.context_bar_bg)
                .fg(app.theme.context_bar_fg),
        );
    f.render_widget(block, area);

    // Left: Brand
    let brand_text = " [ Sisyphus ] ";
    let brand = Span::styled(
        brand_text,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(app.theme.context_bar_fg),
    );

    // Right: Model
    let model_text = format!(" [ {} ] ", app.state.active_model);
    let model = Span::styled(
        model_text.clone(),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(app.theme.context_bar_fg),
    );

    // Center: CWD
    // We need to calculate available width for CWD
    let brand_width = brand_text.len() as u16;
    let model_width = model_text.len() as u16;
    let available_width = area.width.saturating_sub(brand_width + model_width);

    let cwd = &app.state.current_working_directory;
    let cwd_display = truncate_path(cwd, available_width.saturating_sub(2) as usize); // -2 for padding

    let cwd_span = Span::styled(cwd_display, Style::default().fg(app.theme.context_bar_fg));

    // We render 3 paragraphs or 1 paragraph with specific spacing?
    // Paragraph with Left alignment, but we want center and right too.
    // Easier to render 3 separate widgets or construct a single line with spacing.
    // Since we have a solid background, we can just render the text.

    // Left
    let left_para = Paragraph::new(Line::from(brand)).alignment(Alignment::Left);
    f.render_widget(left_para, area);

    // Right
    let right_para = Paragraph::new(Line::from(model)).alignment(Alignment::Right);
    f.render_widget(right_para, area);

    // Center
    let center_para = Paragraph::new(Line::from(cwd_span)).alignment(Alignment::Center);
    f.render_widget(center_para, area);
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    if max_len < 5 {
        return "...".to_string();
    }

    let ellipsis = "...";
    let part_len = (max_len - ellipsis.len()) / 2;

    let start = &path[0..part_len];
    let end = &path[path.len() - part_len..];

    format!("{}{}{}", start, ellipsis, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_path() {
        assert_eq!(
            truncate_path("/home/user/project", 20),
            "/home/user/project"
        );
        assert_eq!(truncate_path("/home/user/project", 10), "/ho...ect");
        assert_eq!(truncate_path("short", 5), "short");
        assert_eq!(truncate_path("longpath", 4), "...");
    }
}
