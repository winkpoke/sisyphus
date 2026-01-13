use colored::*;
use common::config::Config;
use std::env;

pub fn print_startup_info(config: &Config) {
    let logo = r#"
███████ ██ ███████ ██    ██ ███████ ██   ██ ██   ██ ███████
██      ██ ██       ██  ██  ██   ██ ██   ██ ██   ██ ██     
███████ ██ ███████   ████   ███████ ███████ ██   ██ ███████
     ▓▓ ▓▓      ▓▓    ▓▓    ▓▓      ▓▓   ▓▓ ▓▓   ▓▓      ▓▓
▓▓▓▓▓▓▓ ▓▓ ▓▓▓▓▓▓▓    ▓▓    ▓▓      ▓▓   ▓▓ ▓▓▓▓▓▓▓ ▓▓▓▓▓▓▓
"#;

    // Light Gray / Off-White (RGB 200, 200, 200)
    println!("{}", logo.truecolor(200, 200, 200));

    let version = env!("CARGO_PKG_VERSION");
    let current_dir = env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Unknown".to_string());

    let model = &config.llm.model;

    // Box drawing
    let box_width = 60;
    let header = format!(" >_ Sisyphus (v{}) ", version);
    let border_color = Color::White;

    // Top border with header
    // ╭─ HEADER ─...─╮
    let header_len = header.len();
    let right_dash_count = box_width - 2 - 2 - header_len; // 2 corners, 2 dashes left
    let top_border = format!(
        "╭─{}─{}╮",
        header.bold().white(),
        "─".repeat(right_dash_count)
    );
    println!("{}", top_border.color(border_color));

    // Empty line
    println!(
        "{}{}{}",
        "│".color(border_color),
        " ".repeat(box_width - 2),
        "│".color(border_color)
    );

    // Model line
    // │  model:     gpt-4-turbo         ... │
    let model_label = "model:     ";
    let model_value = model;
    let model_line_content = format!("{}{}", model_label.dimmed(), model_value.bright_white());
    // Calculate visible length (without ANSI codes) for padding
    let model_visible_len = model_label.len() + model_value.len();
    let model_padding = box_width - 2 - 2 - model_visible_len; // 2 borders, 2 margin
    println!(
        "{}  {}{}{}",
        "│".color(border_color),
        model_line_content,
        " ".repeat(model_padding),
        "│".color(border_color)
    );

    // Directory line
    let dir_label = "directory: ";
    // Truncate directory if too long
    let max_dir_len = box_width - 2 - 2 - dir_label.len();
    let dir_value = if current_dir.len() > max_dir_len {
        format!(
            "...{}",
            &current_dir[current_dir.len() - (max_dir_len - 3)..]
        )
    } else {
        current_dir.clone()
    };

    let dir_line_content = format!("{}{}", dir_label.dimmed(), dir_value.bright_white());
    let dir_visible_len = dir_label.len() + dir_value.len();
    let dir_padding = box_width - 2 - 2 - dir_visible_len;
    println!(
        "{}  {}{}{}",
        "│".color(border_color),
        dir_line_content,
        " ".repeat(dir_padding),
        "│".color(border_color)
    );

    // Empty line
    println!(
        "{}{}{}",
        "│".color(border_color),
        " ".repeat(box_width - 2),
        "│".color(border_color)
    );

    // Bottom border
    println!(
        "{}{}{}",
        "╰".color(border_color),
        "─".repeat(box_width - 2),
        "╯".color(border_color)
    );

    println!();
    println!(
        "{}",
        "  The struggle itself toward the heights is enough to fill\n  a man's heart."
            .truecolor(150, 150, 150)
            .italic()
    );
    println!(
        "{}",
        "                                           -- Albert Camus"
            .truecolor(150, 150, 150)
            .italic()
    );
    println!();
}
