mod events;
mod state;
mod ui;

use crate::instrument::analyzer::Analysis;
use leetcode_helper::Trace;

/// Launch the interactive TUI trace viewer.
///
/// Takes the generated `trace` and the original `analysis` (for code display).
/// Blocks until the user quits (q/Esc).
pub fn run_tui(trace: &Trace, analysis: &Analysis) -> Result<(), String> {
    // Initialize terminal (raw mode + alternate screen + panic hook)
    let mut terminal = ratatui::init();

    let primary = analysis
        .public_methods
        .first()
        .ok_or("没有找到 public 方法")?;

    let method_start = primary.body_start_line;
    let method_end = primary.body_end_line;

    let mut app_state =
        state::AppState::new(trace, &analysis.code_lines, method_start, method_end);

    // Main event loop
    let result = loop {
        terminal
            .draw(|frame| ui::render(frame, &app_state))
            .map_err(|e| format!("渲染失败: {}", e))?;

        if app_state.should_quit {
            break Ok(());
        }

        events::handle_events(&mut app_state)?;
    };

    // Restore terminal (raw mode off, alternate screen off)
    ratatui::restore();

    result
}
