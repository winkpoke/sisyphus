use crate::tui::app::App;
use crate::tui::state::{InputMode, TranscriptItemKind};
use crate::tui::ui::components::message_block::MessageBlock;
use ratatui::{
    layout::Rect,
    widgets::{Block, Paragraph, Wrap},
    Frame,
};

/// Visibility rule for transcript items: reasoning summaries follow the
/// `/think` toggle (shown by default), raw reasoning is gated behind debug mode.
fn is_item_visible(
    kind: &TranscriptItemKind,
    show_reasoning_summary: bool,
    debug_mode: bool,
) -> bool {
    match kind {
        TranscriptItemKind::ReasoningSummary => show_reasoning_summary,
        TranscriptItemKind::ReasoningRaw => debug_mode,
        _ => true,
    }
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let transcript_block = Block::default();
    let inner_area = transcript_block.inner(area);
    let width = inner_area.width as usize;

    let mut lines = Vec::new();
    for (i, item) in app.state.transcript.items.iter().enumerate() {
        if !is_item_visible(
            &item.kind,
            app.state.show_reasoning_summary,
            app.state.debug_mode,
        ) {
            continue;
        }

        let message_block = MessageBlock::new(
            app,
            &item.content,
            item.kind.clone(),
            item.is_streaming,
            width,
        );

        let mut block_lines = message_block.render_to_lines();

        // Handle selection highlighting
        if app.state.mode == InputMode::Selection
            && Some(i) == app.state.selection.selected_message_index
        {
            for line in &mut block_lines {
                for span in &mut line.spans {
                    span.style = span.style.add_modifier(ratatui::style::Modifier::REVERSED);
                }
            }
        }

        lines.extend(block_lines);
    }

    // Calculate total lines for scrolling
    let total_lines = calculate_total_lines(&lines, width);

    let height = inner_area.height as usize;
    let scroll_y = calculate_scroll_offset(
        total_lines,
        height,
        app.state.transcript.stick_to_bottom,
        app.state.transcript.scroll_offset,
    );

    let transcript = Paragraph::new(lines)
        .block(transcript_block)
        .wrap(Wrap { trim: false }) // Use false to preserve code block indentation
        .scroll((scroll_y, 0));

    f.render_widget(transcript, area);
}

fn calculate_total_lines(lines: &[ratatui::text::Line], width: usize) -> usize {
    let mut total_lines = 0;
    for line in lines {
        let content_len = line.width();
        if width > 0 {
            total_lines += content_len.div_ceil(width);
        } else {
            total_lines += 1;
        }
    }
    total_lines
}

fn calculate_scroll_offset(
    total_lines: usize,
    height: usize,
    stick_to_bottom: bool,
    manual_offset: u16,
) -> u16 {
    if stick_to_bottom {
        if total_lines > height {
            (total_lines - height) as u16
        } else {
            0
        }
    } else {
        manual_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Line;

    #[test]
    fn test_calculate_total_lines() {
        let lines = vec![
            Line::from("Short line"),
            Line::from("A very long line that should wrap around because it is wider than the width provided"),
        ];

        // Width 10
        // "Short line" -> 10 chars -> 1 line
        // "A very long..." -> 86 chars -> ceil(86/10) = 9 lines
        // Total = 10
        assert_eq!(calculate_total_lines(&lines, 10), 10);
    }

    #[test]
    fn test_scroll_stick_to_bottom() {
        assert_eq!(calculate_scroll_offset(20, 10, true, 0), 10);
        assert_eq!(calculate_scroll_offset(5, 10, true, 0), 0);
    }

    #[test]
    fn test_scroll_manual() {
        assert_eq!(calculate_scroll_offset(20, 10, false, 5), 5);
        assert_eq!(calculate_scroll_offset(5, 10, false, 2), 2);
    }

    // ---- Task 7.2: reasoning visibility rules (spec: cli-tui) ----

    #[test]
    fn test_summary_visible_by_default_and_hidden_when_toggled_off() {
        assert!(
            is_item_visible(&TranscriptItemKind::ReasoningSummary, true, false),
            "summaries are shown by default (toggle on)"
        );
        assert!(
            !is_item_visible(&TranscriptItemKind::ReasoningSummary, false, false),
            "/think toggle off hides summaries"
        );
    }

    #[test]
    fn test_raw_reasoning_gated_by_debug_mode() {
        assert!(
            !is_item_visible(&TranscriptItemKind::ReasoningRaw, true, false),
            "raw reasoning must NOT render when debug mode is disabled, even with summaries shown"
        );
        assert!(
            is_item_visible(&TranscriptItemKind::ReasoningRaw, true, true),
            "raw reasoning may render when debug mode is enabled"
        );
    }

    #[test]
    fn test_regular_items_always_visible() {
        for kind in [
            TranscriptItemKind::User,
            TranscriptItemKind::Assistant,
            TranscriptItemKind::System,
            TranscriptItemKind::Error,
        ] {
            assert!(is_item_visible(&kind, false, false));
        }
    }
}
