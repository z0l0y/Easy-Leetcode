use super::{SyntaxTheme, TraceTheme};
use colored::Colorize;
use leetcode_helper::{Trace, TraceDs};
use std::collections::HashSet;

pub fn format_trace(
    trace: &Trace,
    syntax_theme: &SyntaxTheme,
    trace_theme: &TraceTheme,
    color: bool,
) -> String {
    let mut out = String::new();
    let total = trace.steps.len();

    // Header
    out.push_str(&format!(
        "{}\n",
        format_label("执行追踪:", trace_theme.header, color)
    ));
    out.push_str(&format!(
        "{} {}\n",
        format_label("输入:", trace_theme.var_name, color),
        format_value(&trace.input, trace_theme.var_value, color)
    ));

    if let Some(ref algo) = trace.algorithm {
        out.push_str(&format!(
            "{} {}\n",
            format_label("算法:", trace_theme.var_name, color),
            format_value(algo, trace_theme.var_value, color)
        ));
    }

    // Separator line
    out.push_str(&format_separator(trace_theme.separator, color));
    out.push('\n');

    // Steps
    for (idx, step) in trace.steps.iter().enumerate() {
        let step_num = idx + 1;

        // Step header
        let loop_tag = if step.loop_back {
            format!(" {}", format_value("[循环]", trace_theme.loop_back, color))
        } else {
            String::new()
        };
        out.push_str(&format!(
            "  {} [{}]{}\n",
            format_value(
                &format!("Step {}/{}", step_num, total),
                trace_theme.step_number,
                color
            ),
            format_value(&format!("Line {}", step.line), trace_theme.step_number, color),
            loop_tag,
        ));
        out.push_str(&format_sub_separator(trace_theme.separator, color));

        // Code line(s)
        for code_line in step.code.lines() {
            let highlighted = highlight_code_line_local(code_line.trim_end(), syntax_theme);
            out.push_str(&format!(
                "    {} {}\n",
                format_value("→", trace_theme.arrow, color),
                highlighted,
            ));
        }

        // Note
        if let Some(ref note) = step.note {
            out.push_str(&format!(
                "    {} {}\n",
                format_label("说明:", trace_theme.note, color),
                format_value(note, trace_theme.note, color)
            ));
        }

        // Variables
        if !step.vars.is_empty() {
            out.push_str(&format!(
                "    {}\n",
                format_label("变量:", trace_theme.var_name, color)
            ));
            for var in &step.vars {
                let old_part = if let Some(ref old) = var.old {
                    format!(
                        " {}",
                        format_value(
                            &format!("(旧: {})", old),
                            trace_theme.var_old,
                            color
                        )
                    )
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "      {} = {}{}\n",
                    format_value(&var.name, trace_theme.var_name, color),
                    format_value(&var.value, trace_theme.var_value, color),
                    old_part,
                ));
            }
        }

        // Data structures
        if !step.ds.is_empty() {
            for ds in &step.ds {
                let viz = render_ds_viz(ds, trace_theme, color);
                for line in viz.lines() {
                    out.push_str(&format!("    {}\n", line));
                }
            }
        }

        // Result highlight
        if step.is_result {
            out.push_str(&format!(
                "    {}\n",
                format_value(">>> 返回结果 <<<", trace_theme.result, color)
            ));
        }

        out.push('\n');
    }

    // Footer separator
    out.push_str(&format_separator(trace_theme.separator, color));

    out
}

// ─── Formatting helpers ───────────────────────────────────────────────

fn format_label(text: &str, c: colored::Color, color: bool) -> String {
    if color {
        text.color(c).bold().to_string()
    } else {
        text.to_string()
    }
}

fn format_value(text: &str, c: colored::Color, color: bool) -> String {
    if color {
        text.color(c).to_string()
    } else {
        text.to_string()
    }
}

fn format_separator(c: colored::Color, color: bool) -> String {
    let line = "═".repeat(60);
    if color {
        line.color(c).to_string()
    } else {
        line
    }
}

fn format_sub_separator(c: colored::Color, color: bool) -> String {
    let line = format!("  {}", "─".repeat(40));
    if color {
        line.color(c).to_string()
    } else {
        line
    }
}

// ─── Code highlighting (local copy from main.rs) ─────────────────────

#[derive(Copy, Clone)]
enum TokenKind {
    Default,
    Keyword,
    TypeName,
    Function,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
}

fn highlight_code_line_local(line: &str, theme: &SyntaxTheme) -> String {
    let tokens = lex_code_line_local(line);
    let mut out = String::new();
    for (kind, text) in tokens {
        let c = match kind {
            TokenKind::Default => theme.default,
            TokenKind::Keyword => theme.keyword,
            TokenKind::TypeName => theme.type_name,
            TokenKind::Function => theme.function,
            TokenKind::String => theme.string,
            TokenKind::Number => theme.number,
            TokenKind::Comment => theme.comment,
            TokenKind::Operator => theme.operator,
            TokenKind::Punctuation => theme.punctuation,
        };
        out.push_str(&text.color(c).to_string());
    }
    out
}

fn lex_code_line_local(line: &str) -> Vec<(TokenKind, String)> {
    let keywords: HashSet<&'static str> = [
        "if", "else", "for", "while", "do", "switch", "case", "break", "continue", "return",
        "try", "catch", "finally", "throw", "throws", "new", "class", "interface", "enum",
        "public", "private", "protected", "static", "final", "abstract", "extends",
        "implements", "import", "package", "void", "this", "super", "true", "false", "null",
    ]
    .into_iter()
    .collect();
    let type_words: HashSet<&'static str> = [
        "int", "long", "double", "float", "short", "byte", "char", "boolean", "string", "list",
        "arraylist", "map", "hashmap", "set", "hashset", "deque", "queue", "stack", "object",
    ]
    .into_iter()
    .collect();
    let operators: &[char] = &[
        '+', '-', '*', '/', '%', '=', '>', '<', '!', '&', '|', '^', '~', '?', ':',
    ];
    let punctuations: &[char] = &['(', ')', '[', ']', '{', '}', '.', ',', ';'];

    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    let mut in_block_comment = false;
    while i < chars.len() {
        if in_block_comment {
            let start = i;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    in_block_comment = false;
                    break;
                }
                i += 1;
            }
            if in_block_comment {
                i = chars.len();
            }
            tokens.push((TokenKind::Comment, chars[start..i].iter().collect()));
            continue;
        }

        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            tokens.push((TokenKind::Comment, chars[i..].iter().collect()));
            break;
        }
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            in_block_comment = true;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    in_block_comment = false;
                    break;
                }
                i += 1;
            }
            tokens.push((TokenKind::Comment, chars[start..i].iter().collect()));
            continue;
        }

        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            if i > chars.len() {
                i = chars.len();
            }
            tokens.push((TokenKind::String, chars[start..i].iter().collect()));
            continue;
        }

        if chars[i].is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push((TokenKind::Number, chars[start..i].iter().collect()));
            continue;
        }

        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let lower = word.to_ascii_lowercase();

            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            let is_function = j < chars.len() && chars[j] == '(';
            let kind = if keywords.contains(lower.as_str()) {
                TokenKind::Keyword
            } else if type_words.contains(lower.as_str())
                || word.chars().next().is_some_and(|ch| ch.is_ascii_uppercase())
            {
                TokenKind::TypeName
            } else if is_function {
                TokenKind::Function
            } else {
                TokenKind::Default
            };

            tokens.push((kind, word));
            continue;
        }

        if operators.contains(&chars[i]) {
            tokens.push((TokenKind::Operator, chars[i].to_string()));
            i += 1;
            continue;
        }
        if punctuations.contains(&chars[i]) {
            tokens.push((TokenKind::Punctuation, chars[i].to_string()));
            i += 1;
            continue;
        }

        tokens.push((TokenKind::Default, chars[i].to_string()));
        i += 1;
    }
    tokens
}

// ─── Data structure visualization dispatcher ─────────────────────────

fn render_ds_viz(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    // If pre-rendered ascii is provided, use it directly
    if let Some(ref ascii) = ds.ascii {
        let mut out = String::new();
        out.push_str(&format!(
            "{} {}\n",
            format_label(&format!("{}:", ds.label), theme.ds_label, color),
            ""
        ));
        for line in ascii.lines() {
            out.push_str(&format!("      {}\n", format_value(line, theme.var_value, color)));
        }
        // Remove trailing newline
        if out.ends_with('\n') {
            out.pop();
        }
        return out;
    }

    match ds.kind.as_deref() {
        Some("hashmap") => render_hashmap_ds(ds, theme, color),
        Some("stack") => render_stack_ds(ds, theme, color),
        Some("queue") => render_queue_ds(ds, theme, color),
        Some("linkedlist") => render_linkedlist_ds(ds, theme, color),
        Some("window") => render_window_ds(ds, theme, color),
        Some("twopointer") | None => render_array_ds(ds, theme, color),
        _ => render_array_ds(ds, theme, color),
    }
}

// ─── Array / TwoPointer visualization ────────────────────────────────

fn render_array_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return String::new();
    }

    let highlight_set: HashSet<usize> = ds.highlight.as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    let max_w = values.iter().map(|s| s.len()).max().unwrap_or(1);
    let plain_label = format!("{}: ", ds.label);

    // First, build a plain text (no ANSI) version for position calculation
    let mut plain_line = plain_label.clone();
    plain_line.push_str("[ ");
    let mut value_starts: Vec<usize> = Vec::new();

    for (i, val) in values.iter().enumerate() {
        value_starts.push(plain_line.len());
        plain_line.push_str(val);
        plain_line.push_str(&" ".repeat(max_w.saturating_sub(val.len())));
        if i < values.len() - 1 {
            plain_line.push_str(", ");
        }
    }
    plain_line.push_str(" ]");

    // Then build the colored version for display
    let colored_label = format_label(&plain_label, theme.ds_label, color);
    let mut colored_line = colored_label.clone();
    colored_line.push_str("[ ");

    for (i, val) in values.iter().enumerate() {
        let padding = " ".repeat(max_w.saturating_sub(val.len()));
        if highlight_set.contains(&i) {
            colored_line.push_str(&format!(
                "{}{}",
                format_value(val, theme.ds_highlight, color),
                padding,
            ));
        } else {
            colored_line.push_str(&format!(
                "{}{}",
                format_value(val, theme.var_value, color),
                padding,
            ));
        }
        if i < values.len() - 1 {
            colored_line.push_str(", ");
        }
    }
    colored_line.push_str(" ]");

    let mut out = String::new();
    out.push_str(&colored_line);
    out.push('\n');

    let plain_len = plain_line.len();

    // Highlight/pointer line
    let has_highlights = !highlight_set.is_empty();
    let has_ptrs = ds.ptr_left.is_some() || ds.ptr_right.is_some();

    if has_highlights || has_ptrs {
        let mut ptr_line = vec![' '; plain_len];

        // Mark highlight positions using plain text offsets
        for &hi in &highlight_set {
            if hi < value_starts.len() {
                let center = value_starts[hi] + values[hi].len() / 2;
                if center < plain_len {
                    ptr_line[center] = '^';
                }
            }
        }

        // Show highlight index numbers
        for &hi in &highlight_set {
            if hi < value_starts.len() {
                let start = value_starts[hi];
                let idx_str = hi.to_string();
                for (j, ch) in idx_str.chars().enumerate() {
                    let p = start + j;
                    if p < plain_len {
                        ptr_line[p] = ch;
                    }
                }
            }
        }

        let ptr_str: String = ptr_line.iter().collect();
        let trimmed = ptr_str.trim_end();
        if !trimmed.is_empty() {
            out.push_str(&format_value(trimmed, theme.ds_pointer, color));
            out.push('\n');
        }
    }

    // L/R pointer line
    if ds.ptr_left.is_some() || ds.ptr_right.is_some() {
        let mut lr_line = vec![' '; plain_len];

        if let Some(l) = ds.ptr_left {
            if l < value_starts.len() {
                let center = value_starts[l] + values[l].len() / 2;
                if center < plain_len {
                    lr_line[center] = '^';
                }
                let l_pos = (center + 1).min(plain_len - 1);
                if l_pos < plain_len {
                    lr_line[l_pos] = 'L';
                }
            }
        }

        if let Some(r) = ds.ptr_right {
            if r < value_starts.len() {
                let center = value_starts[r] + values[r].len() / 2;
                if center < plain_len {
                    lr_line[center] = '^';
                }
                let r_pos = (center + 1).min(plain_len - 1);
                if r_pos < plain_len {
                    lr_line[r_pos] = 'R';
                }
            }
        }

        let lr_str: String = lr_line.iter().collect();
        let trimmed = lr_str.trim_end();
        if !trimmed.is_empty() {
            out.push_str(&format_value(trimmed, theme.ds_pointer, color));
            out.push('\n');
        }
    }

    // Remove trailing newline
    if out.ends_with('\n') {
        out.pop();
    }

    out
}

// ─── HashMap visualization ────────────────────────────────────────────

fn render_hashmap_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));

    if let Some(ref data) = ds.data {
        if let Some(obj) = data.as_object() {
            let entries: Vec<String> = obj.iter().map(|(k, v)| {
                let key_str = format_value(k, theme.ds_highlight, color);
                let val_str = format_value(&format_key_value(v), theme.var_value, color);
                format!("{}: {}", key_str, val_str)
            }).collect();
            out.push_str(&format!("{{ {} }}", entries.join(", ")));
        } else {
            out.push_str("{}");
        }
    } else {
        out.push_str("{}");
    }

    out
}

// ─── Stack visualization ──────────────────────────────────────────────

fn render_stack_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));

    if values.is_empty() {
        out.push_str("bottom [ ] top");
    } else {
        let joined: Vec<String> = values.iter().map(|v| {
            format_value(v, theme.var_value, color)
        }).collect();
        let items = joined.join(", ");
        out.push_str(&format!("bottom [ {} ] top", items));
    }

    out
}

// ─── Queue visualization ──────────────────────────────────────────────

fn render_queue_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));

    if values.is_empty() {
        out.push_str("front [ ] back");
    } else {
        let joined: Vec<String> = values.iter().map(|v| {
            format_value(v, theme.var_value, color)
        }).collect();
        let items = joined.join(", ");
        out.push_str(&format!("front [ {} ] back", items));
    }

    out
}

// ─── Linked List visualization ────────────────────────────────────────

fn render_linkedlist_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    let highlight_set: HashSet<usize> = ds.highlight.as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));

    if values.is_empty() {
        out.push_str("null");
        return out;
    }

    let joined: Vec<String> = values.iter().enumerate().map(|(i, v)| {
        if highlight_set.contains(&i) {
            format_value(v, theme.ds_highlight, color)
        } else {
            format_value(v, theme.var_value, color)
        }
    }).collect();

    out.push_str(&joined.join(" → "));
    out.push_str(" → null");

    // If there are highlights, add a pointer line underneath
    if !highlight_set.is_empty() {
        out.push('\n');
        // Build pointer line roughly aligned
        let mut ptr_parts = Vec::new();
        let mut offset = ds.label.len() + 2; // ": " after label
        for (i, v) in values.iter().enumerate() {
            let seg_len = v.len() + if i < values.len() - 1 { 3 } else { 0 }; // " → " separator
            if highlight_set.contains(&i) {
                let padding = " ".repeat(offset + v.len() / 2);
                ptr_parts.push(format!(
                    "{}{}",
                    padding,
                    format_value("^cur", theme.ds_pointer, color)
                ));
            }
            offset += seg_len;
        }
        out.push_str(&ptr_parts.join(""));
    }

    out
}

// ─── Sliding Window visualization ─────────────────────────────────────

fn render_window_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return String::new();
    }

    let highlight_set: HashSet<usize> = ds.highlight.as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    let left = ds.ptr_left;
    let right = ds.ptr_right;
    let max_w = values.iter().map(|s| s.len()).max().unwrap_or(1);
    let plain_label = format!("{}: ", ds.label);

    // Build plain text line for position calculation
    let mut plain_line = plain_label.clone();
    let mut value_starts: Vec<usize> = Vec::new();
    plain_line.push_str("[ ");
    for (i, val) in values.iter().enumerate() {
        value_starts.push(plain_line.len());
        plain_line.push_str(val);
        plain_line.push_str(&" ".repeat(max_w.saturating_sub(val.len())));
        if i < values.len() - 1 {
            plain_line.push_str(", ");
        }
    }
    plain_line.push_str(" ]");

    // Build colored line for display
    let colored_label = format_label(&plain_label, theme.ds_label, color);
    let mut colored_line = colored_label.clone();
    colored_line.push_str("[ ");
    for (i, val) in values.iter().enumerate() {
        let padding = " ".repeat(max_w.saturating_sub(val.len()));
        if highlight_set.contains(&i) {
            colored_line.push_str(&format!(
                "{}{}",
                format_value(val, theme.ds_highlight, color),
                padding,
            ));
        } else {
            colored_line.push_str(&format!(
                "{}{}",
                format_value(val, theme.var_value, color),
                padding,
            ));
        }
        if i < values.len() - 1 {
            colored_line.push_str(", ");
        }
    }
    colored_line.push_str(" ]");

    let mut out = String::new();
    out.push_str(&colored_line);
    out.push('\n');

    let plain_len = plain_line.len();

    // Window bracket line
    if let (Some(l), Some(r)) = (left, right) {
        if l < value_starts.len() && r < value_starts.len() {
            let l_center = value_starts[l] + values[l].len() / 2;
            let r_center = value_starts[r] + values[r].len() / 2;

            let mut window_line = vec![' '; plain_len];

            if l_center + 1 < plain_len {
                window_line[l_center] = '<';
                window_line[l_center + 1] = '-';
            }

            let dash_start = if l_center + 2 < plain_len { l_center + 2 } else { l_center + 1 };
            let dash_end = r_center.min(plain_len);
            for p in dash_start..dash_end {
                if window_line[p] == ' ' {
                    window_line[p] = '-';
                }
            }

            if r_center < plain_len {
                window_line[r_center] = '>';
            }

            // Add "window" label
            let mid = (l_center + r_center) / 2;
            let window_label = " window ";
            let wl_start = mid.saturating_sub(window_label.len() / 2);
            for (j, ch) in window_label.chars().enumerate() {
                let p = (wl_start + j).min(plain_len - 1);
                window_line[p] = ch;
            }

            let wl_str: String = window_line.iter().collect();
            let trimmed = wl_str.trim_end();
            if !trimmed.is_empty() {
                out.push_str(&format_value(trimmed, theme.ds_pointer, color));
                out.push('\n');
            }

            // left/right labels
            let mut label_line = vec![' '; plain_len];
            let left_label = format!("left={}", l);
            let left_label_start = l_center.saturating_sub(left_label.len() / 2);
            for (j, ch) in left_label.chars().enumerate() {
                let p = (left_label_start + j).min(plain_len - 1);
                label_line[p] = ch;
            }

            let right_label = format!("right={}", r);
            let right_label_start = r_center.saturating_sub(right_label.len() / 2);
            for (j, ch) in right_label.chars().enumerate() {
                let p = (right_label_start + j).min(plain_len - 1);
                label_line[p] = ch;
            }

            let lbl_str: String = label_line.iter().collect();
            let trimmed_lbl = lbl_str.trim_end();
            if !trimmed_lbl.is_empty() {
                out.push_str(&format_value(trimmed_lbl, theme.ds_pointer, color));
                out.push('\n');
            }
        }
    }

    if out.ends_with('\n') {
        out.pop();
    }

    out
}

// ─── Helpers ──────────────────────────────────────────────────────────

/// Extract string values from a JSON array in TraceDs.data.
fn extract_array_values(data: &Option<serde_json::Value>) -> Vec<String> {
    match data {
        Some(serde_json::Value::Array(arr)) => arr.iter().map(|v| format_key_value(v)).collect(),
        Some(serde_json::Value::Object(obj)) => obj.values().map(|v| format_key_value(v)).collect(),
        Some(other) => vec![format_key_value(other)],
        None => vec![],
    }
}

/// Format a JSON value for display in visualizations.
fn format_key_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_key_value).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(_) => "{...}".to_string(),
    }
}
