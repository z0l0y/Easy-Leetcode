use super::analyzer::Analysis;
use super::example::{ExampleInput, TypedValue};
use anyhow::Context;
use leetcode_helper::{Trace, TraceDs, TraceStep, TraceVar};
use std::collections::HashMap;

/// Parse Java stdout output into a Trace structure.
pub fn parse(
    output_lines: &[String],
    _answer: &str,
    analysis: &Analysis,
    input: &ExampleInput,
) -> anyhow::Result<Trace> {
    let code_lines = &analysis.code_lines;
    let mut steps: Vec<TraceStep> = Vec::new();
    let mut prev_vars: HashMap<String, String> = HashMap::new();
    let mut seen_lines: HashMap<usize, usize> = HashMap::new(); // line -> count for loop detection

    let input_desc = match input {
        ExampleInput::Single(params) => params
            .iter()
            .map(|(name, val)| format!("{} = {}", name, format_val(val)))
            .collect::<Vec<_>>()
            .join(", "),
        ExampleInput::Operations(_, ops) => ops
            .iter()
            .map(|(name, args)| {
                let args_str: Vec<String> = args.iter().map(format_val).collect();
                format!("{}({})", name, args_str.join(", "))
            })
            .collect::<Vec<_>>()
            .join(" → "),
    };

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
            serde_json::from_str(json_str).context("解析 trace JSON 失败")?;

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

        // Parse call stack from JSON
        let call_stack: Vec<String> = parsed
            .get("stack")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| {
                // If no explicit stack, use the method field for single-frame stack
                parsed
                    .get("method")
                    .and_then(|m| m.as_str())
                    .map(|m| vec![m.to_string()])
                    .unwrap_or_default()
            });

        // Generate simple data structure visualizations from variables
        let mut ds: Vec<TraceDs> = Vec::new();
        if let Some(obj) = vars_obj.as_object() {
            // First pass: collect int variables for pointer detection
            let mut pointer_vars: Vec<(String, usize)> = Vec::new();
            for (name, value) in obj {
                let val_str = value.as_str().unwrap_or("");
                if let Ok(n) = val_str.parse::<usize>() {
                    let lower = name.to_ascii_lowercase();
                    let is_pointer = lower == "left" || lower == "right"
                        || lower == "slow" || lower == "fast"
                        || lower == "lo" || lower == "hi"
                        || lower == "l" || lower == "r"
                        || lower == "start" || lower == "end"
                        || lower == "i" || lower == "j";
                    if is_pointer {
                        pointer_vars.push((name.clone(), n));
                    }
                }
            }

            for (name, value) in obj {
                let val_str = value.as_str().unwrap_or("");

                // ── Detect TreeNode toLevelOrder FIRST: [1,2,3,null,4,5] ──
                if val_str.starts_with('[') && val_str.ends_with(']') && val_str.contains("null") {
                    let inner = &val_str[1..val_str.len() - 1];
                    let vals: Vec<serde_json::Value> = inner
                        .split(',')
                        .map(|s| {
                            let s = s.trim();
                            if s == "null" {
                                serde_json::Value::Null
                            } else if let Ok(n) = s.parse::<i64>() {
                                serde_json::Value::Number(n.into())
                            } else {
                                serde_json::Value::String(s.to_string())
                            }
                        })
                        .collect();
                    if vals.iter().any(|v| v.is_null()) {
                        ds.push(TraceDs {
                            kind: Some("tree".into()),
                            label: name.clone(),
                            data: Some(serde_json::Value::Array(vals)),
                            ascii: None,
                            highlight: None,
                            ptr_left: None,
                            ptr_right: None,
                        });
                        continue; // Don't also treat as regular array
                    }
                }

                // ── Detect regular array values ──────────────────────
                if val_str.starts_with('[') && val_str.ends_with(']') {
                    if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(val_str) {
                        // Auto-annotate with pointer positions
                        let mut highlight: Vec<usize> = Vec::new();
                        let mut ptr_left: Option<usize> = None;
                        let mut ptr_right: Option<usize> = None;

                        for (_pname, pval) in &pointer_vars {
                            if *pval < arr.len() {
                                highlight.push(*pval);
                            }
                        }
                        // Use first two pointer vars for left/right
                        if pointer_vars.len() >= 2 {
                            if pointer_vars[0].1 < arr.len() {
                                ptr_left = Some(pointer_vars[0].1);
                            }
                            if pointer_vars[1].1 < arr.len() {
                                ptr_right = Some(pointer_vars[1].1);
                            }
                        } else if pointer_vars.len() == 1 {
                            if pointer_vars[0].1 < arr.len() {
                                ptr_left = Some(pointer_vars[0].1);
                            }
                        }

                        ds.push(TraceDs {
                            kind: Some("array".into()),
                            label: name.clone(),
                            data: Some(serde_json::Value::Array(arr)),
                            ascii: None,
                            highlight: if highlight.is_empty() { None } else { Some(highlight) },
                            ptr_left,
                            ptr_right,
                        });
                    }
                }

                // ── Detect 2D array for heatmap ───────────────────────
                if val_str.starts_with("[[") && val_str.ends_with("]]") {
                    if let Ok(arr2d) = serde_json::from_str::<Vec<Vec<serde_json::Value>>>(val_str) {
                        let flat: Vec<serde_json::Value> = arr2d
                            .into_iter()
                            .map(|row| serde_json::Value::Array(row))
                            .collect();
                        ds.push(TraceDs {
                            kind: Some("heatmap".into()),
                            label: name.clone(),
                            data: Some(serde_json::Value::Array(flat)),
                            ascii: None,
                            highlight: None,
                            ptr_left: None,
                            ptr_right: None,
                        });
                        continue;
                    }
                }

                // Detect HashMap toString: {k=v, k=v}
                if val_str.starts_with('{') && val_str.ends_with('}') && val_str.contains('=') {
                    let inner = &val_str[1..val_str.len() - 1];
                    let mut map = serde_json::Map::new();
                    for pair in inner.split(", ") {
                        if let Some(eq) = pair.find('=') {
                            let k = pair[..eq].trim().to_string();
                            let v = pair[eq + 1..].trim().to_string();
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
                // Detect ListNode toString: 1->2->3->null or 1->2->3
                if val_str.contains("->") && !val_str.starts_with('[') && !val_str.starts_with('{') {
                    let parts: Vec<&str> = val_str.split("->").collect();
                    if parts.len() >= 1 {
                        let vals: Vec<serde_json::Value> = parts
                            .iter()
                            .filter(|p| **p != "null")
                            .map(|p| {
                                if let Ok(n) = p.trim().parse::<i64>() {
                                    serde_json::Value::Number(n.into())
                                } else {
                                    serde_json::Value::String(p.trim().to_string())
                                }
                            })
                            .collect();
                        ds.push(TraceDs {
                            kind: Some("linkedlist".into()),
                            label: name.clone(),
                            data: Some(serde_json::Value::Array(vals)),
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
            call_stack,
        });
    }

    if steps.is_empty() {
        anyhow::bail!("未捕获到任何执行步骤")
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

#[cfg(test)]
mod tests {
    use super::super::analyzer;
    use super::super::example::TypedValue;
    use super::*;

    const TWO_SUM: &str = "import java.util.*;\n\nclass Solution {\n    public int[] twoSum(int[] nums, int target) {\n        Map<Integer, Integer> map = new HashMap<>();\n        for (int i = 0; i < nums.length; i++) {\n            int complement = target - nums[i];\n            if (map.containsKey(complement)) {\n                return new int[]{map.get(complement), i};\n            }\n            map.put(nums[i], i);\n        }\n        return new int[]{};\n    }\n}";

    #[test]
    fn test_parse_trace_lines() {
        let analysis = analyzer::analyze(TWO_SUM).unwrap();
        let input = ExampleInput::Single(vec![
            ("nums".to_string(), TypedValue::Array(vec![TypedValue::Int(2), TypedValue::Int(7)])),
            ("target".to_string(), TypedValue::Int(9)),
        ]);

        // Simulated __TRACE__ output lines from instrumented Java
        let output = vec![
            "__TRACE__{\"line\":5,\"vars\":{\"target\":\"9\",\"nums\":\"[2, 7]\"}}".to_string(),
            "__TRACE__{\"line\":6,\"vars\":{\"i\":\"0\",\"map\":\"{}\",\"nums\":\"[2, 7]\",\"target\":\"9\"}}".to_string(),
            "__TRACE__{\"line\":7,\"vars\":{\"complement\":\"7\",\"i\":\"0\",\"map\":\"{}\",\"nums\":\"[2, 7]\",\"target\":\"9\"}}".to_string(),
        ];

        let trace = parse(&output, TWO_SUM, &analysis, &input).unwrap();
        assert_eq!(trace.steps.len(), 3, "should have 3 steps");
        assert_eq!(trace.steps[0].line, "5");
        assert!(!trace.steps[0].vars.is_empty(), "step should have vars");
        // Input description should contain parameter values
        assert!(trace.input.contains("nums"), "input should mention nums");
    }

    #[test]
    fn test_detect_return_step() {
        let analysis = analyzer::analyze(TWO_SUM).unwrap();
        let input = ExampleInput::Single(vec![
            ("nums".to_string(), TypedValue::Array(vec![TypedValue::Int(2), TypedValue::Int(7)])),
            ("target".to_string(), TypedValue::Int(9)),
        ]);

        let output = vec![
            "__TRACE__{\"line\":9,\"vars\":{\"__return__\":\"[0, 1]\",\"complement\":\"7\",\"i\":\"0\"}}".to_string(),
        ];

        let trace = parse(&output, TWO_SUM, &analysis, &input).unwrap();
        assert_eq!(trace.steps.len(), 1);
        assert!(trace.steps[0].is_result, "return step should be marked as result");
    }

    #[test]
    fn test_linked_list_detection() {
        let list_code = "class Solution {\n    public ListNode reverseList(ListNode head) {\n        ListNode prev = null;\n        ListNode cur = head;\n        while (cur != null) {\n            ListNode next = cur.next;\n            cur.next = prev;\n            prev = cur;\n            cur = next;\n        }\n        return prev;\n    }\n}";
        let analysis = analyzer::analyze(list_code).unwrap();
        let input = ExampleInput::Single(vec![
            ("head".to_string(), TypedValue::String("1->2->3->null".to_string())),
        ]);

        let output = vec![
            "__TRACE__{\"line\":6,\"vars\":{\"prev\":\"1->null\",\"cur\":\"2->3->null\"}}".to_string(),
        ];

        let trace = parse(&output, list_code, &analysis, &input).unwrap();
        // Should have detected linked list DS from the -> pattern
        let has_ll = trace.steps[0].ds.iter().any(|d| d.kind.as_deref() == Some("linkedlist"));
        assert!(has_ll, "should detect linked list from -> pattern");
    }
}
