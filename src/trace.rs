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

// ─── Public plain-text renderer (used by TUI) ────────────────────────

/// Render a TraceDs to plain (uncolored) multi-line ASCII art.
/// Each line is a separate String. Returns empty vec if no data.
pub fn render_ds_plain(ds: &TraceDs) -> Vec<String> {
    // If pre-rendered ascii is provided, use it directly
    if let Some(ref ascii) = ds.ascii {
        return ascii.lines().map(|s| s.to_string()).collect();
    }

    match ds.kind.as_deref() {
        Some("hashmap") => {
            let body = render_hashmap_body(ds);
            if body.is_empty() { vec![] } else { vec![body] }
        }
        Some("stack") | Some("queue") => {
            let body = if ds.kind.as_deref() == Some("stack") {
                render_stack_body(ds)
            } else {
                render_queue_body(ds)
            };
            if body.is_empty() { vec![] } else { vec![body] }
        }
        Some("linkedlist") => render_linkedlist_plain(ds),
        Some("tree") => render_tree_plain(ds),
        Some("heatmap") => render_heatmap_plain(ds),
        Some("window") => render_window_plain(ds),
        _ => {
            // Default: array visualization
            render_array_plain(ds)
        }
    }
}

// ─── Plain (uncolored) renderers (used by both TUI and colored path) ──

fn render_linkedlist_plain(ds: &TraceDs) -> Vec<String> {
    // Compact mini-box format for TUI:  [1]→[2]→[3]→null
    // with pointer annotations and change markers above when available.
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return vec!["null".to_string()];
    }

    let ptrs: Vec<(String, usize)> = ds.ptrs.clone().unwrap_or_default();
    let highlight_set: std::collections::HashSet<usize> = ds
        .highlight
        .as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    // Build the value chain line
    let parts: Vec<String> = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            if highlight_set.contains(&i) {
                format!("[*{}]", v) // mark changed node
            } else {
                format!("[{}]", v)
            }
        })
        .collect();
    let chain = format!("{}→null", parts.join("→"));

    let mut lines: Vec<String> = Vec::new();

    // If there are pointer annotations, build a pointer line above
    if !ptrs.is_empty() {
        // Calculate the center column of each box in the chain
        let mut box_centers: Vec<usize> = Vec::new();
        let mut col = 0usize;
        for part in &parts {
            box_centers.push(col + part.chars().count() / 2);
            col += part.chars().count() + 1; // +1 for '→'
        }

        let total_w = chain.chars().count();
        let mut ptr_line: Vec<char> = vec![' '; total_w];

        for (name, idx) in &ptrs {
            if *idx >= box_centers.len() {
                continue;
            }
            let center = box_centers[*idx];

            // Place down-arrow at box center
            if center < total_w {
                ptr_line[center] = '↓';
            }

            // Place name, centered over the box
            let name_start = center.saturating_sub(name.chars().count() / 2);
            for (j, ch) in name.chars().enumerate() {
                let p = name_start + j;
                if p < total_w {
                    ptr_line[p] = ch;
                }
            }
        }

        let ptr_str: String = ptr_line.into_iter().collect();
        let trimmed = ptr_str.trim_end().to_string();
        if !trimmed.is_empty() {
            lines.push(trimmed);
        }
    }

    lines.push(chain);
    lines
}

fn render_tree_plain(ds: &TraceDs) -> Vec<String> {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return vec![];
    }
    let nodes: Vec<Option<String>> = values
        .iter()
        .map(|v| if v == "null" { None } else { Some(v.clone()) })
        .collect();

    // Build highlight set from the DS
    let highlight_set: std::collections::HashSet<usize> = ds
        .highlight
        .as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    let root = match build_tree_from_level_order(&nodes, &highlight_set) {
        Some(r) => r,
        None => return vec![],
    };
    render_tree_horizontal(&root)
}

/// Rendered subtree result.
struct TreeRender {
    lines: Vec<String>,
    root_col: usize, // column index of the root value start in lines[0]
    width: usize,    // total display width of this subtree
}

/// Render a binary tree as a horizontal diagram using ASCII / and \ branches.
/// This is the standard LeetCode convention and avoids box-drawing alignment issues.
///
/// Example output:
///      3
///     / \
///    9  20
///       / \
///      15  7
///
/// Single child:    5         5
///                 /           \
///                3             8
fn render_tree_horizontal(root: &TreeNode) -> Vec<String> {
    let result = build_tree_render(root);
    result.lines
}

fn build_tree_render(node: &TreeNode) -> TreeRender {
    let val = node.display_val(); // owned String (may include '*' prefix)
    let val_w = node.val_width();

    // Render children
    let left_render = node.left.as_ref().map(|l| build_tree_render(l));
    let right_render = node.right.as_ref().map(|r| build_tree_render(r));

    match (&left_render, &right_render) {
        (None, None) => {
            // Leaf node
            TreeRender { lines: vec![val], root_col: 0, width: val_w }
        }
        (Some(l), None) => {
            // Only left child: root centered just right of the child
            //    root
            //   /
            //  child
            let child_root = l.root_col;
            // Place '/' at child's root position, root right above/beside it
            let slash_pos = child_root;
            let root_col = slash_pos + 1; // root starts just after the slash
            let total_w = (root_col + val_w).max(l.width);

            let mut lines = Vec::new();

            // Root line
            let mut root_line = String::new();
            pad_to(&mut root_line, root_col);
            root_line.push_str(&val);
            pad_to(&mut root_line, total_w);
            lines.push(root_line);

            // Branch line: '/' at slash_pos
            let mut branch = String::new();
            pad_to(&mut branch, slash_pos);
            branch.push('/');
            pad_to(&mut branch, total_w);
            lines.push(branch);

            // Left subtree lines
            for ll in &l.lines {
                let mut line = ll.clone();
                pad_to(&mut line, total_w);
                lines.push(line);
            }

            TreeRender { lines, root_col, width: total_w }
        }
        (None, Some(r)) => {
            // Only right child: root sits left, '\' connects down-right
            //  root
            //   \
            //   child
            let root_col = 0usize;
            let slash_pos = val_w.max(1) - 1; // position of '\' under the root
            let child_offset = slash_pos + 1; // where child subtree starts
            let total_w = child_offset + r.width;

            let mut lines = Vec::new();

            // Root line
            let mut root_line = val;
            pad_to(&mut root_line, total_w);
            lines.push(root_line);

            // Branch line: '\' at slash_pos
            let mut branch = String::new();
            pad_to(&mut branch, slash_pos);
            branch.push('\\');
            pad_to(&mut branch, total_w);
            lines.push(branch);

            // Right subtree lines, shifted right by child_offset
            for rl in &r.lines {
                let mut line = String::new();
                pad_to(&mut line, child_offset);
                line.push_str(rl);
                pad_to(&mut line, total_w);
                lines.push(line);
            }

            TreeRender { lines, root_col, width: total_w }
        }
        (Some(l), Some(r)) => {
            // Both children: root centered, / \ to left and right children
            let gap = 1; // minimum 1 space between the / and \
            let total_w = l.width + gap + 2 + r.width; // gap + "/" + "\" as extra
            // Actually: place '/' at l.root_col, '\' at l.width + 1 + r.root_col
            let left_slash = l.root_col;
            let right_slash = l.width + 1 + gap + r.root_col;
            let actual_w = (right_slash + 1).max(total_w).max(
                l.width + 1 + gap + r.width
            );
            let root_col = (left_slash + right_slash) / 2 + 1 - (val_w + 1) / 2;
            let total_w = (root_col + val_w).max(actual_w);

            let mut lines = Vec::new();

            // Root line
            let mut root_line = String::new();
            pad_to(&mut root_line, root_col);
            root_line.push_str(&val);
            pad_to(&mut root_line, total_w);
            lines.push(root_line);

            // Branch line: '/' at left_slash, '\' at right_slash
            let mut branch = String::new();
            pad_to(&mut branch, left_slash);
            branch.push('/');
            pad_to(&mut branch, right_slash);
            branch.push('\\');
            pad_to(&mut branch, total_w);
            lines.push(branch);

            // Combine subtree lines side by side
            let inner_gap = right_slash - l.width; // gap between left subtree end and right subtree start
            let max_h = l.lines.len().max(r.lines.len());
            for i in 0..max_h {
                let left_part = if i < l.lines.len() { &l.lines[i] } else { "" };
                let right_part = if i < r.lines.len() { &r.lines[i] } else { "" };
                let mut combined = left_part.to_string();
                pad_to(&mut combined, l.width);
                pad_to(&mut combined, l.width + inner_gap);
                combined.push_str(right_part);
                pad_to(&mut combined, total_w);
                lines.push(combined);
            }

            // Recompute root_col accurately
            let root_col_final = if val_w > 0 {
                (left_slash + right_slash) / 2 + 1 - val_w / 2
            } else {
                root_col
            };

            TreeRender { lines, root_col: root_col_final, width: total_w }
        }
    }
}

/// Pad a string with spaces to at least `target_w` display columns.
fn pad_to(s: &mut String, target_w: usize) {
    while s.chars().count() < target_w {
        s.push(' ');
    }
}

fn render_array_plain(ds: &TraceDs) -> Vec<String> {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return vec!["[]".to_string()];
    }

    let highlight_set: HashSet<usize> = ds
        .highlight
        .as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    // Build array display (no label — TUI adds its own)
    let mut line = String::from("[");
    for (i, val) in values.iter().enumerate() {
        if i > 0 {
            line.push(',');
        }
        if highlight_set.contains(&i) {
            line.push_str(&format!("*{}", val));
        } else {
            line.push_str(val);
        }
    }
    line.push(']');

    // Pointer annotations for two-pointer/window (use label as context)
    if let (Some(l), Some(r)) = (ds.ptr_left, ds.ptr_right) {
        if l < values.len() && r < values.len() {
            vec![line, format!("  L={} R={}", l, r)]
        } else {
            vec![line]
        }
    } else {
        vec![line]
    }
}

fn render_hashmap_body(ds: &TraceDs) -> String {
    if let Some(ref data) = ds.data {
        if let Some(obj) = data.as_object() {
            let entries: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_key_value(v)))
                .collect();
            return format!("{{ {} }}", entries.join(", "));
        }
    }
    "{}".to_string()
}

fn render_stack_body(ds: &TraceDs) -> String {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        "bottom [ ] top".to_string()
    } else {
        format!("bottom [ {} ] top", values.join(", "))
    }
}

fn render_queue_body(ds: &TraceDs) -> String {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        "front [ ] back".to_string()
    } else {
        format!("front [ {} ] back", values.join(", "))
    }
}

fn render_heatmap_plain(ds: &TraceDs) -> Vec<String> {
    let rows: Vec<Vec<String>> = match &ds.data {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                v.as_array()
                    .map(|row| row.iter().map(format_key_value).collect())
            })
            .collect(),
        _ => return vec![],
    };
    if rows.is_empty() {
        return vec![];
    }
    let cell_w = rows.iter().flatten().map(|s| s.len()).max().unwrap_or(1) + 1;
    let mut lines = Vec::new();
    for row in &rows {
        let line: String = row
            .iter()
            .map(|val| format!("{:^width$}", val, width = cell_w))
            .collect();
        lines.push(format!("  {}", line));
    }
    lines
}

fn render_window_plain(ds: &TraceDs) -> Vec<String> {
    let values = extract_array_values(&ds.data);
    if values.is_empty() {
        return vec!["[]".to_string()];
    }
    let mut line = String::from("[");
    line.push_str(&values.join(", "));
    line.push(']');

    let mut lines = vec![line];

    if let (Some(l), Some(r)) = (ds.ptr_left, ds.ptr_right) {
        if l < values.len() && r < values.len() {
            let desc = format!("window [{}, {}]  L={} R={}", l, r, l, r);
            lines.push(desc);
        }
    }

    lines
}

// ─── Data structure visualization dispatcher (colored) ────────────────

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

// ─── Linked List visualization (box-drawing style) ──────────────────

fn render_linkedlist_ds(ds: &TraceDs, theme: &TraceTheme, color: bool) -> String {
    let values = extract_array_values(&ds.data);
    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));

    if values.is_empty() {
        out.push_str("null");
        return out;
    }

    out.push('\n');

    let max_w = values.iter().map(|s| s.len()).max().unwrap_or(1);
    let gap = 3usize; // spacing between boxes on top/bottom lines (matches "──▶" width)

    let ptrs: Vec<(String, usize)> = ds.ptrs.clone().unwrap_or_default();

    // Build pointer annotation lines above the boxes
    if !ptrs.is_empty() {
        let (name_line, arrow_line) = build_ll_ptr_lines(&values, max_w, gap, &ptrs);
        if !name_line.is_empty() {
            out.push_str(&format!(
                "      {}\n",
                format_value(&name_line, theme.ds_pointer, color)
            ));
        }
        if !arrow_line.is_empty() {
            out.push_str(&format!(
                "      {}\n",
                format_value(&arrow_line, theme.ds_pointer, color)
            ));
        }
    }

    // Box-drawing lines
    out.push_str(&format!(
        "      {}\n",
        format_value(&build_ll_top(&values, max_w, gap), theme.var_value, color)
    ));
    out.push_str(&format!(
        "      {}\n",
        format_value(&build_ll_mid(&values, max_w), theme.var_value, color)
    ));
    out.push_str(&format!(
        "      {}\n",
        format_value(&build_ll_bot(&values, max_w, gap), theme.var_value, color)
    ));

    // Trim trailing newline
    if out.ends_with('\n') {
        out.pop();
    }

    out
}

/// Build the top border line of linked list box visualization.
fn build_ll_top(values: &[String], max_w: usize, gap: usize) -> String {
    let content_w = max_w + 2; // 1 space padding on each side
    let inner = "─".repeat(content_w);
    let mut line = String::new();
    for (i, _) in values.iter().enumerate() {
        line.push_str(&format!("┌{}┐", inner));
        if i < values.len() - 1 {
            line.push_str(&" ".repeat(gap));
        }
    }
    line.trim_end().to_string()
}

/// Build the value line of linked list box visualization with connectors.
fn build_ll_mid(values: &[String], max_w: usize) -> String {
    let mut line = String::new();
    for (i, v) in values.iter().enumerate() {
        let padded = format!(" {:>width$} ", v, width = max_w);
        line.push_str(&format!("│{}│", padded));
        if i < values.len() - 1 {
            line.push_str("──▶");
        } else {
            line.push_str("──▶ null");
        }
    }
    line
}

/// Build the bottom border line of linked list box visualization.
fn build_ll_bot(values: &[String], max_w: usize, gap: usize) -> String {
    let content_w = max_w + 2;
    let inner = "─".repeat(content_w);
    let mut line = String::new();
    for (i, _) in values.iter().enumerate() {
        line.push_str(&format!("└{}┘", inner));
        if i < values.len() - 1 {
            line.push_str(&" ".repeat(gap));
        }
    }
    line.trim_end().to_string()
}

/// Build pointer annotation lines (name line + arrow line) for linked list.
/// Returns (name_line, arrow_line). Each is trimmed on the right.
fn build_ll_ptr_lines(
    values: &[String],
    max_w: usize,
    gap: usize,
    ptrs: &[(String, usize)],
) -> (String, String) {
    let content_w = max_w + 2;
    let box_w = 2 + content_w;
    let stride = box_w + gap;
    let total_w = values.len() * stride - gap; // subtract last gap

    // Sanity check
    if total_w == 0 {
        return (String::new(), String::new());
    }

    let mut name_line: Vec<char> = vec![' '; total_w];
    let mut arrow_line: Vec<char> = vec![' '; total_w];

    for (ptr_name, idx) in ptrs {
        if *idx >= values.len() {
            continue;
        }
        let center = idx * stride + box_w / 2;
        if center >= total_w {
            continue;
        }

        // Place down-arrow at node center
        arrow_line[center] = '▼';

        // Place pointer name centered above the node
        let name_start = center.saturating_sub(ptr_name.len() / 2);
        for (j, ch) in ptr_name.chars().enumerate() {
            let pos = name_start + j;
            if pos < total_w {
                name_line[pos] = ch;
            }
        }
    }

    let name_str: String = name_line.into_iter().collect();
    let arrow_str: String = arrow_line.into_iter().collect();
    let name_trimmed = name_str.trim_end().to_string();
    let arrow_trimmed = arrow_str.trim_end().to_string();
    (name_trimmed, arrow_trimmed)
}

// ─── Tree visualization ──────────────────────────────────────────────

/// A node in the render tree.
struct TreeNode {
    val: String,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    highlighted: bool,
}

impl TreeNode {
    fn display_val(&self) -> String {
        if self.highlighted {
            format!("*{}", self.val)
        } else {
            self.val.clone()
        }
    }

    fn val_width(&self) -> usize {
        if self.highlighted {
            self.val.chars().count() + 1 // +1 for '*'
        } else {
            self.val.chars().count()
        }
    }
}

/// Build a binary tree from a level-order array with optional highlight set.
fn build_tree_from_level_order(
    vals: &[Option<String>],
    highlight_set: &std::collections::HashSet<usize>,
) -> Option<Box<TreeNode>> {
    if vals.is_empty() || vals[0].is_none() {
        return None;
    }
    let mut nodes: Vec<Option<Box<TreeNode>>> = vals
        .iter()
        .enumerate()
        .map(|(i, v)| v.as_ref().map(|s| Box::new(TreeNode {
            val: s.clone(),
            left: None,
            right: None,
            highlighted: highlight_set.contains(&i),
        })))
        .collect();

    // Process in reverse: when we take() a child, its own children
    // (at higher indices) are already linked.
    for i in (0..vals.len()).rev() {
        if nodes[i].is_none() {
            continue;
        }
        let left_idx = 2 * i + 1;
        let right_idx = 2 * i + 2;
        if left_idx >= vals.len() && right_idx >= vals.len() {
            continue;
        }
        let (left_part, right_part) = nodes.split_at_mut(i + 1);
        let node = left_part[i].as_mut().unwrap();
        if left_idx < vals.len() {
            let child_idx = left_idx - (i + 1);
            if child_idx < right_part.len() {
                node.left = right_part[child_idx].take();
            }
        }
        if right_idx < vals.len() {
            let child_idx = right_idx - (i + 1);
            if child_idx < right_part.len() {
                node.right = right_part[child_idx].take();
            }
        }
    }
    nodes.into_iter().next().flatten()
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

    // Build highlight set to mark changed nodes
    let highlight_set: std::collections::HashSet<usize> = ds
        .highlight
        .as_ref()
        .map(|v| v.iter().cloned().collect())
        .unwrap_or_default();

    // Build tree from level-order array
    let root = match build_tree_from_level_order(&nodes, &highlight_set) {
        Some(r) => r,
        None => return String::new(),
    };

    // Render as horizontal tree diagram with box-drawing characters
    let tree_lines = render_tree_horizontal(&root);

    let mut out = String::new();
    out.push_str(&format!(
        "{} ",
        format_label(&format!("{}:", ds.label), theme.ds_label, color)
    ));
    out.push('\n');

    for line in &tree_lines {
        out.push_str(&format!("      {}\n", format_value(line, theme.var_value, color)));
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
