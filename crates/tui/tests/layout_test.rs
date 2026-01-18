use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tui::tui::app::App;
use tui::tui::ui;

#[test]
fn test_layout_rendering() {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new("test-session".to_string());

    terminal
        .draw(|f| {
            ui::draw(f, &app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    
    // Check for ContextBar elements
    // We expect " SISYPHUS " in the header (ContextBar)
    // The exact position depends on the layout, but it should be there.
    // " SISYPHUS " is in bold, possibly with specific colors.
    
    // Simple check: iterate over cells and look for content
    let mut found_title = false;
    for y in 0..30 {
        let line_text: String = (0..100)
            .map(|x| buffer.get(x, y).symbol().to_string())
            .collect();
        if line_text.contains("Sisyphus") {
            found_title = true;
            break;
        }
    }
    assert!(found_title, "ContextBar title not found in rendered output");

    // Check for Input prompt "> "
    let mut found_prompt = false;
    for y in 0..30 {
        let line_text: String = (0..100)
            .map(|x| buffer.get(x, y).symbol().to_string())
            .collect();
        if line_text.contains("> ") {
            found_prompt = true;
            break;
        }
    }
    assert!(found_prompt, "Input prompt not found in rendered output");
}

#[test]
fn test_small_terminal_warning() {
    let backend = TestBackend::new(30, 5); // Critically small (needs 40x10)
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new("test-session".to_string());

    terminal
        .draw(|f| {
            ui::draw(f, &app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    
    let mut found_warning = false;
    for y in 0..5 {
        let line_text: String = (0..30)
            .map(|x| buffer.get(x, y).symbol().to_string())
            .collect();
        if line_text.contains("Terminal too small") {
            found_warning = true;
            break;
        }
    }
    assert!(found_warning, "Warning message not found in critically small terminal");
}

#[test]
fn test_degraded_mode() {
    let backend = TestBackend::new(60, 20); // Degraded (needs 80x24 for full, but > 40x10)
    let mut terminal = Terminal::new(backend).unwrap();
    let app = App::new("test-session".to_string());

    terminal
        .draw(|f| {
            ui::draw(f, &app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    
    // Should NOT have "Terminal too small"
    let mut found_warning = false;
    for y in 0..20 {
        let line_text: String = (0..60)
            .map(|x| buffer.get(x, y).symbol().to_string())
            .collect();
        if line_text.contains("Terminal too small") {
            found_warning = true;
            break;
        }
    }
    assert!(!found_warning, "Warning message should not appear in degraded mode");

    // Should NOT have ContextBar ("Sisyphus")
    let mut found_title = false;
    for y in 0..20 {
        let line_text: String = (0..60)
            .map(|x| buffer.get(x, y).symbol().to_string())
            .collect();
        if line_text.contains("Sisyphus") {
            found_title = true;
            break;
        }
    }
    assert!(!found_title, "ContextBar should be hidden in degraded mode");

    // Should HAVE Input prompt "> "
    let mut found_prompt = false;
    for y in 0..20 {
        let line_text: String = (0..60)
            .map(|x| buffer.get(x, y).symbol().to_string())
            .collect();
        if line_text.contains("> ") {
            found_prompt = true;
            break;
        }
    }
    assert!(found_prompt, "Input prompt should be visible in degraded mode");
}
