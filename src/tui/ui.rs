use super::state::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

// ─── Color constants ──────────────────────────────────────────────────

const COLOR_LINE_NO: Color = Color::Gray;
const COLOR_CUR_LINE: Color = Color::Rgb(40, 44, 52); // dark bg highlight
const COLOR_CUR_MARKER: Color = Color::Rgb(97, 175, 239); // blue arrow
const COLOR_VAR_NAME: Color = Color::Rgb(97, 175, 239);
const COLOR_VAR_VALUE: Color = Color::Rgb(229, 192, 123);
const COLOR_CHANGED: Color = Color::Rgb(229, 192, 123);
const COLOR_TITLE: Color = Color::Rgb(152, 195, 121);
const COLOR_BORDER: Color = Color::Rgb(92, 99, 112);
const COLOR_STATUS: Color = Color::Rgb(171, 178, 191);
const COLOR_RESULT: Color = Color::Rgb(152, 195, 121);

// ─── Main render entry ─────────────────────────────────────────────────

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    // Split vertically: main content + status bar
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let main_area = vchunks[0];
    let status_area = vchunks[1];

    // Split main horizontally: code (60%) | watch (40%)
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_area);

    let code_area = hchunks[0];
    let watch_area = hchunks[1];

    // Auto-scroll before rendering
    let _visible_lines = code_area.height.saturating_sub(2) as usize; // minus borders
    // (can't mutate state in immutable render, so we won't autoscroll here)

    render_code_panel(frame, code_area, state);
    render_watch_panel(frame, watch_area, state);
    render_status_bar(frame, status_area, state);
}

// ─── Code panel ────────────────────────────────────────────────────────

fn render_code_panel(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let cur_line = state.current_line();
    let method_lines = state.visible_code_slice();
    let start_line = state.method_start; // 1-indexed first line of method

    let mut lines: Vec<Line> = Vec::new();

    for (i, code) in method_lines.iter().enumerate() {
        let line_no = start_line + i;
        let is_current = line_no == cur_line;

        let ln_str = format!("{:>3} ", line_no);
        let ln_span = if is_current {
            Span::styled(ln_str, Style::default().fg(COLOR_CUR_MARKER).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(ln_str, Style::default().fg(COLOR_LINE_NO))
        };

        let code_span = highlight_java(code, is_current);

        let mut line = Line::default();
        line.push_span(ln_span);
        line.push_span(code_span);

        if is_current {
            // Highlight the entire line background
            line = line.style(Style::default().bg(COLOR_CUR_LINE));
            // Insert marker arrow
            line.spans.insert(0, Span::styled("▶ ", Style::default().fg(COLOR_CUR_MARKER).add_modifier(Modifier::BOLD)));
        } else {
            line = line.style(Style::default());
        }

        lines.push(line);
    }

    // Scroll
    let scroll = state.scroll_offset;
    let visible = area.height.saturating_sub(2) as usize; // minus borders
    let start = scroll.min(lines.len().saturating_sub(visible));
    let end = (start + visible).min(lines.len());
    let sliced: Vec<Line> = lines[start..end].to_vec();

    let paragraph = Paragraph::new(Text::from(sliced))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .title(Span::styled(" Code ", Style::default().fg(COLOR_TITLE).add_modifier(Modifier::BOLD))),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

// ─── Watch panel ───────────────────────────────────────────────────────

fn render_watch_panel(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_vars_table(frame, chunks[0], state);
    render_ds_view(frame, chunks[1], state);
}

fn render_vars_table(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let step = match state.current() {
        Some(s) => s,
        None => {
            let p = Paragraph::new("(无数据)")
                .block(Block::default().borders(Borders::ALL)
                    .title(" Variables "));
            frame.render_widget(p, area);
            return;
        }
    };

    let header = Row::new(vec![
        Cell::from(Span::styled("Name", Style::default().add_modifier(Modifier::BOLD))),
        Cell::from(Span::styled("Value", Style::default().add_modifier(Modifier::BOLD))),
    ])
    .style(Style::default());

    let rows: Vec<Row> = step
        .vars
        .iter()
        .map(|v| {
            let is_return = v.name == "__return__";
            let changed = v.old.is_some();

            let name_style = if is_return {
                Style::default().fg(COLOR_RESULT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_VAR_NAME)
            };
            let val_style = if is_return {
                Style::default().fg(COLOR_RESULT).add_modifier(Modifier::BOLD)
            } else if changed {
                Style::default().fg(COLOR_CHANGED).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(COLOR_VAR_VALUE)
            };

            let display_name = if is_return {
                "Return Value".to_string()
            } else {
                v.name.clone()
            };

            let mut val_text = v.value.clone();
            if let Some(ref old) = v.old {
                val_text.push_str(&format!("  (was {})", old));
            }

            Row::new(vec![
                Cell::from(Span::styled(display_name, name_style)),
                Cell::from(Span::styled(val_text, val_style)),
            ])
        })
        .collect();

    let title = format!(" Variables (step {}/{}) ", state.current_step + 1, state.total_steps());
    let table = Table::new(rows, [Constraint::Percentage(40), Constraint::Percentage(60)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .title(Span::styled(title, Style::default().fg(COLOR_TITLE).add_modifier(Modifier::BOLD))),
        )
        .column_spacing(1);

    frame.render_widget(table, area);
}

fn render_ds_view(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let step = match state.current() {
        Some(s) => s,
        None => {
            let p = Paragraph::new("(无数据)")
                .block(Block::default().borders(Borders::ALL).title(" Data "));
            frame.render_widget(p, area);
            return;
        }
    };

    let mut lines: Vec<Line> = Vec::new();

    if step.ds.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no data structures)",
            Style::default().fg(COLOR_LINE_NO),
        )));
    } else {
        for ds in &step.ds {
            // Simple visualization of data structures
            let label = Span::styled(
                format!("{}: ", ds.label),
                Style::default().fg(COLOR_TITLE).add_modifier(Modifier::BOLD),
            );
            let body = format_ds_value(ds);
            let mut line = Line::default();
            line.push_span(label);
            line.push_span(Span::styled(body, Style::default().fg(COLOR_VAR_VALUE)));
            lines.push(line);
        }
    }

    // If this is a result step, add a highlight
    if step.is_result {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            ">>> RESULT <<<",
            Style::default().fg(COLOR_RESULT).add_modifier(Modifier::BOLD),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(COLOR_BORDER))
                .title(Span::styled(" Data ", Style::default().fg(COLOR_TITLE).add_modifier(Modifier::BOLD))),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

// ─── Status bar ────────────────────────────────────────────────────────

fn render_status_bar(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let step = state.current();
    let loop_tag = if step.map(|s| s.loop_back).unwrap_or(false) {
        " [循环]"
    } else {
        ""
    };
    let result_tag = if step.map(|s| s.is_result).unwrap_or(false) {
        " [结果]"
    } else {
        ""
    };

    let text = format!(
        " Step {}/{} (Line {}){}{} │ Enter/→:next  ←:prev  g:first  G:last  q:quit │ ↑↓ PgUp/PgDn:scroll ",
        state.current_step + 1,
        state.total_steps(),
        state.current_line(),
        loop_tag,
        result_tag,
    );

    let p = Paragraph::new(Span::styled(text, Style::default().fg(COLOR_STATUS)))
        .style(Style::default().bg(Color::Rgb(33, 37, 43)));
    frame.render_widget(p, area);
}

// ─── Helpers ───────────────────────────────────────────────────────────

/// Simple Java syntax highlighting returning a Span.
fn highlight_java(code: &str, is_current: bool) -> Span<'_> {
    let base_style = if is_current {
        Style::default().fg(Color::Rgb(220, 220, 220))
    } else {
        Style::default().fg(Color::Rgb(171, 178, 191))
    };

    // Very basic highlighting — just color keywords differently.
    // The trimmed code gets colored, keeping original indentation.
    let trimmed = code.trim_end();
    Span::styled(trimmed.to_string(), base_style)
}

/// Format a TraceDs value for display in the data panel.
fn format_ds_value(ds: &leetcode_helper::TraceDs) -> String {
    match &ds.data {
        Some(serde_json::Value::Array(arr)) => {
            let items: Vec<String> = arr.iter().map(format_json_val).collect();
            format!("[{}]", items.join(", "))
        }
        Some(serde_json::Value::Object(obj)) => {
            let items: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_json_val(v)))
                .collect();
            format!("{{ {} }}", items.join(", "))
        }
        Some(other) => format_json_val(other),
        None => String::new(),
    }
}

fn format_json_val(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_json_val).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(_) => "{...}".to_string(),
    }
}
