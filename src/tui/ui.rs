use super::state::AppState;
use crate::highlight;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

// ─── Main render entry ─────────────────────────────────────────────────

pub fn render(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();

    // Source bar (1) | code+watch (flex) | status bar (1)
    let vchunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let source_area = vchunks[0];
    let main_area = vchunks[1];
    let status_area = vchunks[2];

    // Split main horizontally: code (60%) | watch (40%)
    let hchunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_area);

    let code_area = hchunks[0];
    let watch_area = hchunks[1];

    render_source_bar(frame, source_area, state);
    render_code_panel(frame, code_area, state);
    render_watch_panel(frame, watch_area, state);
    render_status_bar(frame, status_area, state);
}

// ─── Source bar ─────────────────────────────────────────────────────────

fn render_source_bar(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let text = format!(" Source: {} ", state.trace.input);
    let p = Paragraph::new(Span::styled(
        text,
        Style::default()
            .fg(state.tui_theme.title)
            .add_modifier(Modifier::BOLD),
    ))
    .style(Style::default().bg(state.tui_theme.source_bar_bg));
    frame.render_widget(p, area);
}

// ─── Code panel ────────────────────────────────────────────────────────

fn render_code_panel(frame: &mut Frame, area: ratatui::layout::Rect, state: &mut AppState) {
    // Write back visible lines for autoscroll (P0-6)
    let visible = area.height.saturating_sub(2) as usize;
    state.visible_lines = visible.max(1);

    let cur_line = state.current_line();
    let method_lines = state.visible_code_slice();
    let start_line = state.method_start;

    let mut lines: Vec<Line> = Vec::new();

    for (i, code) in method_lines.iter().enumerate() {
        let line_no = start_line + i;
        let is_current = line_no == cur_line;
        let has_bp = state.breakpoints.contains(&line_no);

        let ln_str = if has_bp {
            format!("●{:>2} ", line_no)
        } else {
            format!(" {:>3} ", line_no)
        };
        let ln_style = if is_current {
            Style::default()
                .fg(state.tui_theme.cur_marker)
                .add_modifier(Modifier::BOLD)
        } else if has_bp {
            Style::default()
                .fg(Color::Rgb(224, 108, 117))
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(state.tui_theme.line_no)
        };
        let ln_span = Span::styled(ln_str, ln_style);

        let code_style = if is_current {
            Style::default().fg(Color::Rgb(220, 220, 220))
        } else {
            Style::default().fg(Color::Rgb(171, 178, 191))
        };
        let code_span = Span::styled(code.trim_end().to_string(), code_style);

        let mut line = Line::default();
        line.push_span(ln_span);
        line.push_span(code_span);

        if is_current {
            line = line.style(Style::default().bg(state.tui_theme.cur_line_bg));
            line.spans.insert(
                0,
                Span::styled(
                    "▶ ",
                    Style::default()
                        .fg(state.tui_theme.cur_marker)
                        .add_modifier(Modifier::BOLD),
                ),
            );
        } else {
            line = line.style(Style::default());
        }

        lines.push(line);
    }

    // Scroll
    let scroll = state.scroll_offset;
    let start = scroll.min(lines.len().saturating_sub(visible));
    let end = (start + visible).min(lines.len());
    let sliced: Vec<Line> = lines[start..end].to_vec();

    let paragraph = Paragraph::new(Text::from(sliced))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.tui_theme.border))
                .title(Span::styled(
                    " Code ",
                    Style::default()
                        .fg(state.tui_theme.title)
                        .add_modifier(Modifier::BOLD),
                )),
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
            let p = Paragraph::new("(无数据)").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Variables "),
            );
            frame.render_widget(p, area);
            return;
        }
    };

    let header = Row::new(vec![
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Value",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ])
    .style(Style::default());

    let rows: Vec<Row> = step
        .vars
        .iter()
        .map(|v| {
            let is_return = v.name == "__return__";
            let changed = v.old.is_some();

            // Check search match
            let name_matches = state.search_active
                && !state.search_query.is_empty()
                && v.name
                    .to_ascii_lowercase()
                    .contains(&state.search_query.to_ascii_lowercase());

            let name_style = if is_return {
                Style::default()
                    .fg(state.tui_theme.result)
                    .add_modifier(Modifier::BOLD)
            } else if name_matches {
                Style::default()
                    .fg(Color::Rgb(229, 192, 123))
                    .bg(Color::Rgb(80, 60, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(state.tui_theme.var_name)
            };

            // P0-5: changed values get background highlight + bold
            let val_style = if is_return {
                Style::default()
                    .fg(state.tui_theme.result)
                    .add_modifier(Modifier::BOLD)
            } else if changed {
                Style::default()
                    .fg(state.tui_theme.changed)
                    .bg(state.tui_theme.changed_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(state.tui_theme.var_value)
            };

            let display_name = if is_return {
                "Return Value".to_string()
            } else {
                v.name.clone()
            };

            // P0-4: truncate long values, normalize hashmap = → :
            let display_val = normalize_var_value(&v.value);
            let val_text = if let Some(ref old) = v.old {
                let display_old = normalize_var_value(old);
                format!("{} → {}", highlight::truncate_value(&display_old, 30), highlight::truncate_value(&display_val, 60))
            } else {
                highlight::truncate_value(&display_val, 60)
            };

            Row::new(vec![
                Cell::from(Span::styled(display_name, name_style)),
                Cell::from(Span::styled(val_text, val_style)),
            ])
        })
        .collect();

    let title = format!(
        " Variables (step {}/{}) ",
        state.current_step + 1,
        state.total_steps()
    );
    let table = Table::new(rows, [Constraint::Percentage(40), Constraint::Percentage(60)])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.tui_theme.border))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(state.tui_theme.title)
                        .add_modifier(Modifier::BOLD),
                )),
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

    // Show call stack if not empty
    if !step.call_stack.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("📞 调用栈: {}", step.call_stack.join(" → ")),
            Style::default()
                .fg(state.tui_theme.title)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
    }

    if step.ds.is_empty() {
        lines.push(Line::from(Span::styled(
            "(no data structures)",
            Style::default().fg(state.tui_theme.line_no),
        )));
    } else {
        for ds in &step.ds {
            // Label line
            let label = Span::styled(
                format!("┌ {} ", ds.label),
                Style::default()
                    .fg(state.tui_theme.title)
                    .add_modifier(Modifier::BOLD),
            );
            lines.push(Line::from(label));

            // Body lines from the shared plain renderer
            let body_lines = format_ds_lines(ds);
            if body_lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "│ (empty)",
                    Style::default().fg(state.tui_theme.line_no),
                )));
            } else {
                for bl in &body_lines {
                    let trimmed = bl.trim_end();
                    if trimmed.is_empty() {
                        continue;
                    }
                    lines.push(Line::from(Span::styled(
                        format!("│ {}", trimmed),
                        Style::default().fg(state.tui_theme.var_value),
                    )));
                }
            }
            // Separator between data structures
            lines.push(Line::from(""));
        }
        // Remove trailing empty line
        if lines.last().map(|l| l.width() == 0).unwrap_or(false) {
            lines.pop();
        }
    }

    // If this is a result step, add a highlight
    if step.is_result {
        if !lines.is_empty() && lines.last().map(|l| l.width() != 0).unwrap_or(false) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            ">>> RESULT <<<",
            Style::default()
                .fg(state.tui_theme.result)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(state.tui_theme.border))
                .title(Span::styled(
                    " Data ",
                    Style::default()
                        .fg(state.tui_theme.title)
                        .add_modifier(Modifier::BOLD),
                )),
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
    let running_tag = if state.running { " [Running...]" } else { "" };
    let bp_tag = if !state.breakpoints.is_empty() {
        format!(" bp:{}", state.breakpoints.len())
    } else {
        String::new()
    };

    let search_text = if state.search_mode {
        format!(" /{}", state.search_query)
    } else if state.search_active {
        format!(" [search: /{}]", state.search_query)
    } else {
        String::new()
    };

    let text = format!(
        " Step {}/{} (Line {}){}{}{}{}{} │ Enter/→:next  ←:prev  b:bp  c:continue  /:search  g/G:jump  q:quit │ ↑↓ PgUp/PgDn:scroll ",
        state.current_step + 1,
        state.total_steps(),
        state.current_line(),
        loop_tag,
        result_tag,
        running_tag,
        bp_tag,
        search_text,
    );

    let p = Paragraph::new(Span::styled(
        text,
        Style::default().fg(state.tui_theme.status),
    ))
    .style(Style::default().bg(state.tui_theme.status_bar_bg));
    frame.render_widget(p, area);
}

// ─── Helpers ───────────────────────────────────────────────────────────

/// Render a TraceDs to plain (uncolored) lines using the shared plain renderer.
fn format_ds_lines(ds: &leetcode_helper::TraceDs) -> Vec<String> {
    use crate::trace::render_ds_plain;
    render_ds_plain(ds)
}

/// Normalize variable values for display:
/// - Convert HashMap `=` to `: ` (e.g. `{2=0}` → `{2: 0}`)
fn normalize_var_value(raw: &str) -> String {
    // Detect HashMap.toString format: {k=v, k=v} or {k=v}
    if raw.starts_with('{') && raw.ends_with('}') && raw.contains('=') && !raw.contains(" = ") {
        let inner = &raw[1..raw.len() - 1];
        if inner.is_empty() {
            return raw.to_string();
        }
        let pairs: Vec<String> = inner
            .split(", ")
            .map(|pair| {
                if let Some(eq) = pair.find('=') {
                    format!("{}: {}", &pair[..eq].trim(), &pair[eq + 1..].trim())
                } else {
                    pair.to_string()
                }
            })
            .collect();
        format!("{{ {} }}", pairs.join(", "))
    } else {
        raw.to_string()
    }
}
