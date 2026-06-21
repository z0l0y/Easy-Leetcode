use super::state::AppState;
use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

/// Read one keyboard event and update the app state.
pub fn handle_events(state: &mut AppState) -> anyhow::Result<()> {
    // If in "running" mode, keep stepping until breakpoint or end
    if state.running {
        for _ in 0..1000 {
            // max burst to avoid infinite loop in event handler
            if !state.running_step() {
                break;
            }
        }
        return Ok(());
    }

    let ev = event::read().context("读取事件失败")?;
    if let Event::Key(key) = ev {
        // Ignore key release events (they duplicate press events)
        if key.kind == KeyEventKind::Release {
            return handle_events(state); // skip, read next
        }

        // Search mode: grab characters until Enter or Esc
        if state.search_mode {
            handle_search_key(key, state);
            return Ok(());
        }

        handle_key(key, state);
    }
    Ok(())
}

fn handle_search_key(key: crossterm::event::KeyEvent, state: &mut AppState) {
    use KeyCode::*;

    match key.code {
        Esc => {
            state.search_mode = false;
            state.search_query.clear();
            state.search_active = false;
        }
        Enter => {
            state.search_mode = false;
            state.search_active = !state.search_query.is_empty();
        }
        Backspace => {
            state.search_query.pop();
        }
        Char(ch) => {
            state.search_query.push(ch);
        }
        _ => {}
    }
}

fn handle_key(key: crossterm::event::KeyEvent, state: &mut AppState) {
    use KeyCode::*;

    match key.code {
        // ── Step navigation ──────────────────────────────
        Enter | Right | Char(' ') | Char('j') => {
            if state.running {
                state.running = false;
            } else {
                state.next_step();
            }
        }
        Left | Char('k') => state.prev_step(),
        Home | Char('g') => state.first_step(),
        End | Char('G') => state.last_step(),

        // ── Breakpoints ──────────────────────────────────
        Char('b') => state.toggle_breakpoint(),
        Char('c') => {
            if !state.running {
                state.continue_to_breakpoint();
            } else {
                state.running = false;
            }
        }

        // ── Search ───────────────────────────────────────
        Char('/') => {
            state.search_mode = true;
            state.search_query.clear();
            state.search_active = false;
        }

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
