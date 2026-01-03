use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::ui::tui::app::App;
use crate::ui::tui::state::{InputMode, TranscriptItemKind};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let transcript_block = Block::default()
        .title(app.state.context_title.clone())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border))
        .padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
    
    let inner_area = transcript_block.inner(area);
    let width = inner_area.width as usize;

    let mut lines = Vec::new();
    for (i, item) in app.state.transcript.items.iter().enumerate() {
        let prefix = match item.kind {
            TranscriptItemKind::User => "You: ",
            TranscriptItemKind::Assistant => "Assistant: ",
            TranscriptItemKind::System => "System: ",
            TranscriptItemKind::Error => "Error: ",
        };
        let mut style = match item.kind {
            TranscriptItemKind::User => Style::default().fg(app.theme.user),
            TranscriptItemKind::Assistant => Style::default().fg(app.theme.assistant),
            TranscriptItemKind::System => Style::default().fg(app.theme.system),
            TranscriptItemKind::Error => Style::default().fg(app.theme.error),
        };

        if app.state.mode == InputMode::Selection
            && Some(i) == app.state.selection.selected_message_index
        {
            style = style.add_modifier(Modifier::REVERSED);
        }

        let content = item.content.clone();
        let sub_lines: Vec<&str> = content.split('\n').collect();
        
        for (j, sub_line) in sub_lines.iter().enumerate() {
            let mut line_text = String::new();
            if j == 0 {
                line_text.push_str(prefix);
            }
            line_text.push_str(sub_line);

            if item.is_streaming && j == sub_lines.len() - 1 {
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let frame = app.state.spinner_frame % spinner_chars.len();
                line_text.push_str(spinner_chars[frame]);
            }
            
            lines.push(Line::from(Span::styled(line_text, style)));
        }
    }

    // Calculate total lines for scrolling
    let mut total_lines = 0;
    for line in &lines {
        let content_len = line.width();
        if width > 0 {
            total_lines += content_len.div_ceil(width);
        } else {
            total_lines += 1;
        }
    }

    let height = inner_area.height as usize;
    let scroll_y = if app.state.transcript.stick_to_bottom {
        if total_lines > height {
            (total_lines - height) as u16
        } else {
            0
        }
    } else {
        app.state.transcript.scroll_offset
    };

    let transcript = Paragraph::new(lines)
        .block(transcript_block)
        .wrap(Wrap { trim: true })
        .scroll((scroll_y, 0));

    f.render_widget(transcript, area);
}
