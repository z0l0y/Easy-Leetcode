use super::analyzer::Analysis;
use super::example::TypedValue;
use leetcode_helper::{Trace, TraceDs, TraceStep, TraceVar};
use std::collections::HashMap;

/// Parse Java stdout output into a Trace structure.
pub fn parse(
    output_lines: &[String],
    _answer: &str,
    analysis: &Analysis,
    params: &[(String, TypedValue)],
) -> Result<Trace, String> {
    let code_lines = &analysis.code_lines;
    let mut steps: Vec<TraceStep> = Vec::new();
    let mut prev_vars: HashMap<String, String> = HashMap::new();
    let mut seen_lines: HashMap<usize, usize> = HashMap::new(); // line -> count for loop detection

    let input_desc = params
        .iter()
        .map(|(name, val)| format!("{} = {}", name, format_val(val)))
        .collect::<Vec<_>>()
        .join(", ");

    let primary = analysis.public_methods.first();
    let algorithm = primary
        .map(|m| format!("{}.{}", analysis.class_name, m.name))
        .unwrap_or_else(|| "未知算法".to_string());

    for line in output_lines {
        let line = line.trim();

        // Skip non-trace lines
        if !line.starts_with("__TRACE__") {
            continue;
        }

        let json_str = &line["__TRACE__".len()..];

        // Parse the JSON
        let parsed: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("解析 trace JSON 失败: {}", e))?;

        let step_line: usize = parsed["line"]
            .as_u64()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0);

        let vars_obj = &parsed["vars"];

        // Build trace vars list
        let mut trace_vars: Vec<TraceVar> = Vec::new();
        if let Some(obj) = vars_obj.as_object() {
            for (name, value) in obj {
                let val_str = value.as_str().unwrap_or("?").to_string();
                let old = prev_vars.get(name).cloned();
                let changed = old.as_ref().map_or(true, |o| o != &val_str);

                trace_vars.push(TraceVar {
                    name: name.clone(),
                    value: val_str.clone(),
                    old: if changed { old } else { None },
                });

                if changed {
                    prev_vars.insert(name.clone(), val_str);
                }
            }
        }

        // Get the code text for this line
        let code = if step_line > 0 && step_line <= code_lines.len() {
            code_lines[step_line - 1].clone()
        } else {
            String::new()
        };

        // Detect loop_back
        let count = seen_lines.entry(step_line).or_insert(0);
        let loop_back = *count > 0;
        *count += 1;

        // Detect if this is a result step (return line)
        let is_result = code.trim().starts_with("return ") || code.trim().starts_with("return;");

        // Generate simple data structure visualizations from variables
        let mut ds: Vec<TraceDs> = Vec::new();
        if let Some(obj) = vars_obj.as_object() {
            for (name, value) in obj {
                let val_str = value.as_str().unwrap_or("");
                // Try to detect array-like values
                if val_str.starts_with('[') && val_str.ends_with(']') {
                    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(val_str) {
                        ds.push(TraceDs {
                            kind: Some("array".into()),
                            label: name.clone(),
                            data: Some(serde_json::Value::Array(arr)),
                            ascii: None,
                            highlight: None,
                            ptr_left: None,
                            ptr_right: None,
                        });
                    }
                }
                // Detect HashMap toString: {k=v, k=v}
                if val_str.starts_with('{') && val_str.ends_with('}') && val_str.contains('=') {
                    // Convert Java Map.toString to JSON object
                    let inner = &val_str[1..val_str.len() - 1];
                    let mut map = serde_json::Map::new();
                    for pair in inner.split(", ") {
                        if let Some(eq) = pair.find('=') {
                            let k = pair[..eq].trim().to_string();
                            let v = pair[eq + 1..].trim().to_string();
                            // Store as string values
                            map.insert(k, serde_json::Value::String(v));
                        }
                    }
                    if !map.is_empty() {
                        ds.push(TraceDs {
                            kind: Some("hashmap".into()),
                            label: name.clone(),
                            data: Some(serde_json::Value::Object(map)),
                            ascii: None,
                            highlight: None,
                            ptr_left: None,
                            ptr_right: None,
                        });
                    }
                }
            }
        }

        steps.push(TraceStep {
            line: step_line.to_string(),
            code,
            note: None,
            loop_back,
            vars: trace_vars,
            ds,
            is_result,
        });
    }

    if steps.is_empty() {
        return Err("未捕获到任何执行步骤".into());
    }

    Ok(Trace {
        input: input_desc,
        algorithm: Some(algorithm),
        steps,
    })
}

fn format_val(v: &TypedValue) -> String {
    match v {
        TypedValue::Int(n) => n.to_string(),
        TypedValue::String(s) => format!("\"{}\"", s),
        TypedValue::Bool(b) => b.to_string(),
        TypedValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(format_val).collect();
            format!("[{}]", items.join(", "))
        }
        TypedValue::NestedArray(rows) => {
            let rows_str: Vec<String> = rows
                .iter()
                .map(|row| {
                    let items: Vec<String> = row.iter().map(format_val).collect();
                    format!("[{}]", items.join(", "))
                })
                .collect();
            format!("[{}]", rows_str.join(", "))
        }
        TypedValue::TreeNodeArray(elems) => {
            let items: Vec<String> = elems
                .iter()
                .map(|v| match v {
                    Some(n) => n.to_string(),
                    None => "null".to_string(),
                })
                .collect();
            format!("[{}]", items.join(", "))
        }
    }
}
