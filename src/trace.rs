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

        // Call stack
        if !step.call_stack.is_empty() {
            let stack_display = step.call_stack.join(" → ");
            out.push_str(&format!(
                "    {} {}\n",
                format_label("调用栈:", trace_theme.note, color),
                format_value(&stack_display, trace_theme.var_name, color)
            ));
        }

        // Code line(s)
        for code_line in step.code.lines() {
            let highlighted = crate::highlight::highlight_code_line(code_line.trim_end(), syntax_theme, &mut false);
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
                let is_return = var.name == "__return__";
                let name_display = if is_return {
                    "返回值".to_string()
                } else {
                    var.name.clone()
                };
                let name_color = if is_return {
                    trace_theme.result
                } else {
                    trace_theme.var_name
                };
                out.push_str(&format!(
                    "      {} = {}{}\n",
                    format_value(&name_display, name_color, color),
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
        Some("tree") => render_tree_ds(ds, theme, color),
        Some("heatmap") => render_heatmap_ds(ds, theme, color),
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

// ─── Tree visualization ──────────────────────────────────────────────

/// A node in the render tree.
struct TreeNode {
    val: String,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

/// Build a binary tree from a level-order array (Some = node, None = null).
fn build_tree_from_level_order(vals: &[Option<String>]) -> Option<Box<TreeNode>> {
    if vals.is_empty() || vals[0].is_none() {
        return None;
    }
    let mut nodes: Vec<Option<Box<TreeNode>>> = vals
        .iter()
        .map(|v| v.as_ref().map(|s| Box::new(TreeNode {
            val: s.clone(),
            left: None,
            right: None,
        })))
        .collect();

    for i in 0..vals.len() {
        if nodes[i].is_some() {
            let left_idx = 2 * i + 1;
            let right_idx = 2 * i + 2;
            if left_idx < vals.len() {
                nodes[i].as_mut().unwrap().left = nodes[left_idx].take();
            }
            if right_idx < vals.len() {
                nodes[i].as_mut().unwrap().right = nodes[right_idx].take();
            }
        }
    }
    nodes.into_iter().next().flatten()
}

/// Recursive ascii tree rendering. Returns (lines, root_position, width).
fn render_tree_node(node: &TreeNode) -> (Vec<String>, usize, usize) {
    let val = &node.val;
    let val_w = val.len();

    match (&node.left, &node.right) {
        (None, None) => {
            // Leaf node
            (vec![val.clone()], 0, val_w)
        }
        (Some(l), None) => {
            let (left_lines, l_root, l_w) = render_tree_node(l);
            let total_w = l_w.max(val_w);

            let mut lines = Vec::new();
            // Root line
            let l_pad = total_w.saturating_sub(val_w) / 2;
            let mut root_line = " ".repeat(l_pad);
            root_line.push_str(val);
            root_line.push_str(&" ".repeat(total_w.saturating_sub(root_line.len())));
            lines.push(root_line);

            // Connector
            let l_pos = l_root;
            let root_center = l_pad + val_w / 2;
            let mut conn = String::new();
            for i in 0..total_w {
                if i == l_pos {
                    conn.push('┌');
                } else if i == root_center {
                    conn.push('┘');
                } else if i > l_pos.min(root_center) && i < l_pos.max(root_center) {
                    conn.push('─');
                } else {
                    conn.push(' ');
                }
            }
            lines.push(conn);

            // Left subtree lines
            for ll in &left_lines {
                let mut line = ll.clone();
                while line.len() < total_w {
                    line.push(' ');
                }
                lines.push(line);
            }

            (lines, l_pad + val_w / 2, total_w)
        }
        (None, Some(r)) => {
            let (right_lines, r_root, r_w) = render_tree_node(r);
            let total_w = r_w.max(val_w);

            let mut lines = Vec::new();
            // Root line
            let l_pad = total_w.saturating_sub(val_w) / 2;
            let mut root_line = " ".repeat(l_pad);
            root_line.push_str(val);
            root_line.push_str(&" ".repeat(total_w.saturating_sub(root_line.len())));
            lines.push(root_line);

            // Connector
            let r_pos = r_root;
            let root_center = l_pad + val_w / 2;
            let mut conn = String::new();
            for i in 0..total_w {
                if i == root_center {
                    conn.push('└');
                } else if i == r_pos {
                    conn.push('┐');
                } else if i > root_center.min(r_pos) && i < root_center.max(r_pos) {
                    conn.push('─');
                } else {
                    conn.push(' ');
                }
            }
            lines.push(conn);

            // Right subtree lines
            for rl in &right_lines {
                let mut line = rl.clone();
                while line.len() < total_w {
                    line.push(' ');
                }
                lines.push(line);
            }

            (lines, l_pad + val_w / 2, total_w)
        }
        (Some(l), Some(r)) => {
            let (left_lines, l_root, l_w) = render_tree_node(l);
            let (right_lines, r_root, r_w) = render_tree_node(r);
            // Add spacing between subtrees
            let gap = 3usize;
            let total_w = l_w + gap + r_w;

            let mut lines = Vec::new();

            // Root line: center the root value over the two subtrees
            let root_center = l_w + gap / 2;
            let root_start = root_center.saturating_sub(val_w / 2);
            let mut root_line = " ".repeat(root_start);
            root_line.push_str(val);
            root_line.push_str(&" ".repeat(total_w.saturating_sub(root_line.len())));
            lines.push(root_line);

            // Connector line: ┌──┴──┐
            let mut conn = String::new();
            for i in 0..total_w {
                if i == l_root {
                    conn.push('┌');
                } else if i == root_center {
                    conn.push('┴');
                } else if i == l_w + gap + r_root {
                    conn.push('┐');
                } else if (i > l_root && i < root_center) || (i > root_center && i < l_w + gap + r_root) {
                    conn.push('─');
                } else {
                    conn.push(' ');
                }
            }
            lines.push(conn);

            // Combine subtree lines side by side
            let max_h = left_lines.len().max(right_lines.len());
            for i in 0..max_h {
                let left_part = if i < left_lines.len() { &left_lines[i] } else { "" };
                let right_part = if i < right_lines.len() { &right_lines[i] } else { "" };

                let l_padded = format!("{:width$}", left_part, width = l_w);
                let r_padded = format!("{:width$}", right_part, width = r_w);
                let combined = format!("{}{}{}", l_padded, " ".repeat(gap), r_padded);
                lines.push(combined);
            }

            (lines, root_center, total_w)
        }
    }
}

fn render_tree_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return String::new();
    }

    // Parse values into Option<String>
    let nodes: Vec<Option<String>> = values.iter().map(|v| {
        if v == "null" { None } else { Some(v.clone()) }
    }).collect();

    // Build tree from level-order array
    let root = match build_tree_from_level_order(&nodes) {
        Some(r) => r,
        None => return String::new(),
    };

    // Render recursively
    let (tree_lines, _, _) = render_tree_node(&root);

    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));
    out.push('\n');

    for line in &tree_lines {
        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            out.push_str(&format!("      {}\n", format_value(trimmed, theme.var_value, color)));
        }
    }

    // Trim trailing newline
    if out.ends_with('\n') {
        out.pop();
    }

    out
}

// ─── DP Heatmap visualization ────────────────────────────────────────

fn render_heatmap_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));
    out.push('\n');

    // Each element in data should be a row array
    let rows: Vec<Vec<String>> = match &ds.data {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_array().map(|row| row.iter().map(format_key_value).collect()))
            .collect(),
        _ => return out,
    };

    if rows.is_empty() {
        return out;
    }

    // Find min/max for color scaling
    let all_vals: Vec<f64> = rows
        .iter()
        .flatten()
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    let (min_v, max_v) = if all_vals.is_empty() {
        (0.0, 1.0)
    } else {
        let min = all_vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = all_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    };

    let range = if (max_v - min_v).abs() < 1e-9 { 1.0 } else { max_v - min_v };

    // Cell width: max value width + padding
    let cell_w = rows.iter().flatten().map(|s| s.len()).max().unwrap_or(1) + 1;

    for row in &rows {
        let mut line = String::new();
        line.push_str("      ");
        for val in row {
            let num: f64 = val.parse().unwrap_or(0.0);
            let t = ((num - min_v) / range).clamp(0.0, 1.0);

            // Blue (cold) → Red (hot) gradient using ANSI 256-color
            let ansi_code = value_to_heat_color(t);
            let padded = format!("{:^width$}", val, width = cell_w);

            if color {
                line.push_str(&format!("\x1b[48;5;{}m{}\x1b[0m", ansi_code, padded));
            } else {
                line.push_str(&padded);
            }
        }
        out.push_str(&line);
        out.push('\n');
    }

    // Color scale bar
    if color {
        out.push_str("      ");
        let scale_chars = "▁▂▃▄▅▆▇█";
        for (i, ch) in scale_chars.chars().enumerate() {
            let t = i as f64 / (scale_chars.len() - 1) as f64;
            let ansi = value_to_heat_color(t);
            out.push_str(&format!("\x1b[48;5;{}m{}\x1b[0m", ansi, ch));
        }
        out.push_str(&format!(
            " {} {}",
            format_value(&format!("{:.0}", min_v), theme.var_value, color),
            format_value(&format!("{:.0}", max_v), theme.var_value, color)
        ));
        out.push('\n');
    }

    // Trim trailing newline
    if out.ends_with('\n') {
        out.pop();
    }

    out
}

/// Map a value in [0, 1] to an ANSI 256-color heatmap code.
/// 0 → deep blue (17), 0.5 → green (46), 1 → bright red (196)
fn value_to_heat_color(t: f64) -> u8 {
    let t = t.clamp(0.0, 1.0);
    // ANSI 256 color: 16 + (r * 36) + (g * 6) + b
    if t < 0.25 {
        // blue → cyan
        let g = (t / 0.25 * 5.0) as u8;
        16 + (g * 6) + 5
    } else if t < 0.5 {
        // cyan → green
        let b = 5u8.saturating_sub(((t - 0.25) / 0.25 * 5.0) as u8);
        16 + (5 * 6) + b
    } else if t < 0.75 {
        // green → yellow
        let r = ((t - 0.5) / 0.25 * 5.0) as u8;
        16 + (r * 36) + (5 * 6)
    } else {
        // yellow → red
        let g = 5u8.saturating_sub(((t - 0.75) / 0.25 * 5.0) as u8);
        16 + (5 * 36) + (g * 6)
    }
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
