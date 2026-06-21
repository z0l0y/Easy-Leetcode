mod events;
mod state;
mod ui;

use crate::instrument::analyzer::Analysis;
use crate::TuiTheme;
use anyhow::Context;
use leetcode_helper::Trace;

/// Launch the interactive TUI trace viewer.
///
/// Takes the generated `trace`, the original `analysis` (for code display),
/// and the `tui_theme` for color configuration.
/// Blocks until the user quits (q/Esc).
pub fn run_tui(trace: &Trace, analysis: &Analysis, tui_theme: &TuiTheme) -> anyhow::Result<()> {
    // Initialize terminal (raw mode + alternate screen + panic hook)
    let mut terminal = ratatui::init();

    let primary = analysis
        .public_methods
        .first()
        .context("没有找到 public 方法")?;

    let method_start = primary.body_start_line;
    let method_end = primary.body_end_line;

    let mut app_state =
        state::AppState::new(trace, &analysis.code_lines, method_start, method_end, tui_theme);

    // Main event loop
    let result = loop {
        terminal
            .draw(|frame| ui::render(frame, &mut app_state))
            .context("渲染失败")?;

        if app_state.should_quit {
            break Ok(());
        }

        events::handle_events(&mut app_state)?;
    };

    // Restore terminal (raw mode off, alternate screen off)
    ratatui::restore();

    result
}
