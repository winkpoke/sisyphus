use ratatui::style::Color;

pub struct Theme {
    pub user: Color,
    pub assistant: Color,
    pub system: Color,
    pub success: Color,
    pub error: Color,
    pub border: Color,
    pub highlight: Color,
    pub reasoning: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            user: Color::Rgb(93, 173, 226),       // #5dade2 Soft Blue
            assistant: Color::Rgb(165, 105, 189), // #a569bd Lavender
            system: Color::Rgb(128, 139, 150),    // #808b96 Muted Grey
            success: Color::Rgb(46, 204, 113),    // #2ecc71 Bright Green
            error: Color::Rgb(231, 76, 60),       // #e74c3c Soft Red
            border: Color::DarkGray,
            highlight: Color::Yellow,
            reasoning: Color::Rgb(100, 100, 100), // Dark Grey for thinking
        }
    }
}
