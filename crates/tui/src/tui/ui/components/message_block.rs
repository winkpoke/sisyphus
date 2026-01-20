use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::tui::app::App;
use crate::tui::state::TranscriptItemKind;

pub struct MessageBlock<'a> {
    app: &'a App,
    content: &'a str,
    kind: TranscriptItemKind,
    is_streaming: bool,
    width: usize,
}

impl<'a> MessageBlock<'a> {
    pub fn new(
        app: &'a App,
        content: &'a str,
        kind: TranscriptItemKind,
        is_streaming: bool,
        width: usize,
    ) -> Self {
        Self {
            app,
            content,
            kind,
            is_streaming,
            width,
        }
    }

    pub fn render_to_lines(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        // 1. Header
        let (label, color) = match self.kind {
            TranscriptItemKind::User => (" User ", self.app.theme.user),
            TranscriptItemKind::Assistant => (" Assistant ", self.app.theme.assistant),
            TranscriptItemKind::System => (" System ", self.app.theme.system),
            TranscriptItemKind::Error => (" Error ", self.app.theme.error),
            TranscriptItemKind::ReasoningSummary => return self.render_reasoning_summary(),
            TranscriptItemKind::ReasoningRaw => (" Debug ", self.app.theme.system),
        };

        let label_span = Span::styled(
            format!("[{}]", label),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        );

        let label_len = label.len() + 2;
        let separator_char = match self.kind {
            TranscriptItemKind::User => "─",
            TranscriptItemKind::Assistant => "╌",
            TranscriptItemKind::System => "·",
            _ => "-",
        };

        let separator_len = self.width.saturating_sub(label_len + 1);
        let separator = separator_char.repeat(separator_len);
        let separator_span = Span::styled(separator, Style::default().fg(self.app.theme.border));

        lines.push(Line::from(vec![label_span, Span::raw(" "), separator_span]));

        // 2. Content
        let style = if matches!(self.kind, TranscriptItemKind::Error) {
            Style::default().fg(self.app.theme.error)
        } else {
            Style::default().fg(Color::Reset)
        };

        let sub_lines: Vec<&str> = self.content.split('\n').collect();
        for (j, sub_line) in sub_lines.iter().enumerate() {
            let mut line_text = sub_line.to_string();

            if self.is_streaming && j == sub_lines.len() - 1 {
                let spinner_chars = ["⠋", "⠙", "⠹", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let frame = self.app.state.spinner_frame % spinner_chars.len();
                line_text.push_str(spinner_chars[frame]);
            }

            lines.push(Line::from(Span::styled(line_text, style)));
        }

        // 3. Footer/Separator
        lines.push(Line::raw(""));

        lines
    }

    fn render_reasoning_summary(&self) -> Vec<Line<'a>> {
        let mut lines = Vec::new();
        let base_style = Style::default().fg(self.app.theme.reasoning);
        let header_style = base_style.add_modifier(Modifier::BOLD);
        let content_style = base_style.add_modifier(Modifier::ITALIC);

        lines.push(Line::from(Span::styled(
            "  💭 Thinking Process:",
            header_style,
        )));

        let sub_lines: Vec<&str> = self.content.split('\n').collect();
        for (j, sub_line) in sub_lines.iter().enumerate() {
            let mut line_text = format!("  │ {}", sub_line);

            if self.is_streaming && j == sub_lines.len() - 1 {
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let frame = self.app.state.spinner_frame % spinner_chars.len();
                line_text.push_str(spinner_chars[frame]);
            }

            lines.push(Line::from(Span::styled(line_text, content_style)));
        }
        lines.push(Line::raw(""));
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::App;
    use crate::tui::state::TranscriptItemKind;

    #[test]
    fn test_render_user_message() {
        let app = App::new("test-session".to_string());
        let content = "Hello, world!";
        let kind = TranscriptItemKind::User;
        let width = 80;
        let block = MessageBlock::new(&app, content, kind, false, width);
        let lines = block.render_to_lines();

        // Check header
        assert_eq!(lines.len(), 3); // Header, content, footer
        let header = &lines[0];
        assert_eq!(header.spans[0].content, "[ User ]");
        assert_eq!(header.spans[0].style.fg, Some(app.theme.user));

        // Check content
        let content_line = &lines[1];
        assert_eq!(content_line.spans[0].content, "Hello, world!");

        // Check footer
        assert_eq!(lines[2].width(), 0);
    }

    #[test]
    fn test_render_assistant_message() {
        let app = App::new("test-session".to_string());
        let content = "I am Sisyphus.";
        let kind = TranscriptItemKind::Assistant;
        let width = 80;
        let block = MessageBlock::new(&app, content, kind, false, width);
        let lines = block.render_to_lines();

        assert_eq!(lines[0].spans[0].content, "[ Assistant ]");
        assert_eq!(lines[0].spans[0].style.fg, Some(app.theme.assistant));

        // Check separator (Assistant uses "╌")
        let separator = &lines[0].spans[2];
        assert!(separator.content.contains("╌"));
    }

    #[test]
    fn test_render_streaming_spinner() {
        let mut app = App::new("test-session".to_string());
        app.state.spinner_frame = 0;
        let content = "Generating...";
        let kind = TranscriptItemKind::Assistant;
        let width = 80;
        let block = MessageBlock::new(&app, content, kind, true, width);
        let lines = block.render_to_lines();

        // The spinner char at index 0 is "⠋"
        let content_line = &lines[1];
        assert!(content_line.spans[0].content.ends_with("⠋"));
    }
}
