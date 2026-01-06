use crate::ui::tui::app::App;
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let input_block = Block::default().title("Input").borders(Borders::ALL);
    let input_text = Paragraph::new(app.state.input_buffer.clone()).block(input_block);
    f.render_widget(input_text, area);
}
