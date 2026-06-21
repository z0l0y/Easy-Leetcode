use super::state::AppState;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

/// Read one keyboard event and update the app state.
pub fn handle_events(state: &mut AppState) -> Result<(), String> {
    let ev = event::read().map_err(|e| format!("读取事件失败: {}", e))?;
    if let Event::Key(key) = ev {
        // Ignore key release events (they duplicate press events)
        if key.kind == KeyEventKind::Release {
            return handle_events(state); // skip, read next
        }
        handle_key(key, state);
    }
    Ok(())
}

fn handle_key(key: crossterm::event::KeyEvent, state: &mut AppState) {
    use KeyCode::*;

    match key.code {
        // ── Step navigation ──────────────────────────────
        Enter | Right | Char(' ') | Char('j') => state.next_step(),
        Left | Char('k') => state.prev_step(),
        Home | Char('g') => state.first_step(),
        End | Char('G') => state.last_step(),

        // ── Quit ─────────────────────────────────────────
        Char('q') | Esc => state.should_quit = true,

        // ── Scroll code panel ────────────────────────────
        PageDown | Char('J') => {
            state.scroll_offset = state.scroll_offset.saturating_add(8);
        }
        PageUp | Char('K') => {
            state.scroll_offset = state.scroll_offset.saturating_sub(8);
        }
        Down => {
            state.scroll_offset = state.scroll_offset.saturating_add(1);
        }
        Up => {
            state.scroll_offset = state.scroll_offset.saturating_sub(1);
        }

        _ => {}
    }
}
