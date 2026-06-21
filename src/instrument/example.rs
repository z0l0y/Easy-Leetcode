use anyhow::Context;
use serde::Serialize;

/// Typed value parsed from an example input string.
#[derive(Debug, Clone, Serialize)]
pub enum TypedValue {
    Int(i64),
    String(String),
    Bool(bool),
    Array(Vec<TypedValue>),
    NestedArray(Vec<Vec<TypedValue>>),
    TreeNodeArray(Vec<Option<i64>>),
}

impl TypedValue {
    /// Format this value as a Java literal for code generation.
    pub fn to_java_literal(&self) -> String {
        match self {
            TypedValue::Int(n) => n.to_string(),
            TypedValue::String(s) => format!("\"{}\"", s),
            TypedValue::Bool(b) => b.to_string(),
            TypedValue::Array(elems) => {
                let items: Vec<String> = elems.iter().map(|v| v.to_java_literal()).collect();
                format!("{{{}}}", items.join(", "))
            }
            TypedValue::NestedArray(rows) => {
                let row_strs: Vec<String> = rows
                    .iter()
                    .map(|row| {
                        let items: Vec<String> =
                            row.iter().map(|v| v.to_java_literal()).collect();
                        format!("{{{}}}", items.join(", "))
                    })
                    .collect();
                format!("{{{}}}", row_strs.join(", "))
            }
            TypedValue::TreeNodeArray(elems) => {
                // Generate tree construction code
                let items: Vec<String> = elems
                    .iter()
                    .map(|v| match v {
                        Some(n) => n.to_string(),
                        None => "null".to_string(),
                    })
                    .collect();
                format!("new Integer[]{{{}}}", items.join(", "))
            }
        }
    }

    /// Get the Java type name for this value.
    #[allow(dead_code)]
    pub fn java_type(&self) -> &str {
        match self {
            TypedValue::Int(_) => "int",
            TypedValue::String(_) => "String",
            TypedValue::Bool(_) => "boolean",
            TypedValue::Array(elems) => {
                if elems.is_empty() {
                    return "int[]";
                }
                match &elems[0] {
                    TypedValue::Int(_) => "int[]",
                    TypedValue::String(_) => "String[]",
                    _ => "int[]",
                }
            }
            TypedValue::NestedArray(_) => "int[][]",
            TypedValue::TreeNodeArray(_) => "Integer[]",
        }
    }

    /// Generate a Java variable declaration for this value (default int type).
    #[allow(dead_code)]
    pub fn to_java_init(&self, var_name: &str) -> String {
        self.to_java_init_typed(var_name, "int")
    }

    /// Generate a Java variable declaration, using the expected Java type to
    /// produce wrapper code (e.g. buildList / buildTree) for custom types.
    pub fn to_java_init_typed(&self, var_name: &str, expected_type: &str) -> String {
        match self {
            TypedValue::Int(n) => format!("int {} = {};", var_name, n),
            TypedValue::String(s) => format!("String {} = \"{}\";", var_name, s),
            TypedValue::Bool(b) => format!("boolean {} = {};", var_name, b),
            TypedValue::Array(elems) => {
                let lit = self.to_java_literal();
                if expected_type == "ListNode" {
                    // int[] → ListNode via buildList
                    format!("ListNode {} = buildList(new int[]{});", var_name, lit)
                } else if expected_type == "TreeNode" {
                    // int[] → TreeNode via buildTree (already treats nulls)
                    let tree_items: Vec<String> = elems
                        .iter()
                        .map(|v| match v {
                            TypedValue::Int(n) => n.to_string(),
                            _ => "null".to_string(),
                        })
                        .collect();
                    format!(
                        "Integer[] __{}_vals = {{{}}};\n        TreeNode {} = buildTree(__{}_vals);",
                        var_name,
                        tree_items.join(", "),
                        var_name,
                        var_name
                    )
                } else if expected_type == "ListNode[]" {
                    // Array of ListNodes
                    let items: Vec<String> = elems
                        .iter()
                        .map(|v| format!("buildList(new int[]{})", v.to_java_literal()))
                        .collect();
                    format!("ListNode[] {} = {{{}}};", var_name, items.join(", "))
                } else {
                    format!("int[] {} = {};", var_name, lit)
                }
            }
            TypedValue::NestedArray(rows) => {
                if expected_type == "ListNode[]" {
                    let items: Vec<String> = rows
                        .iter()
                        .map(|row| {
                            let lit = TypedValue::Array(row.clone()).to_java_literal();
                            format!("buildList(new int[]{})", lit)
                        })
                        .collect();
                    format!("ListNode[] {} = {{{}}};", var_name, items.join(", "))
                } else if expected_type == "char[][]" {
                    // Generate char[][] with char literals
                    let row_strs: Vec<String> = rows
                        .iter()
                        .map(|row| {
                            let items: Vec<String> = row
                                .iter()
                                .map(|v| {
                                    match v {
                                        TypedValue::String(s) => {
                                            // Take the first char
                                            let ch = s.chars().next().unwrap_or(' ');
                                            format!("'{}'", ch)
                                        }
                                        other => other.to_java_literal(),
                                    }
                                })
                                .collect();
                            format!("{{{}}}", items.join(", "))
                        })
                        .collect();
                    format!("char[][] {} = {{{}}};", var_name, row_strs.join(", "))
                } else if expected_type == "String[][]" {
                    let lit = self.to_java_literal();
                    format!("String[][] {} = {};", var_name, lit)
                } else {
                    let lit = self.to_java_literal();
                    format!("int[][] {} = {};", var_name, lit)
                }
            }
            TypedValue::TreeNodeArray(_elems) => {
                let lit = self.to_java_literal();
                format!(
                    "Integer[] __{}_vals = {};\n        TreeNode {} = buildTree(__{}_vals);",
                    var_name, lit, var_name, var_name
                )
            }
        }
    }
}

/// Represents parsed example input — either a single method call or operation sequence.
#[derive(Debug, Clone)]
pub enum ExampleInput {
    /// Single method: vec of (param_name, value) pairs
    Single(Vec<(String, TypedValue)>),
    /// Operation sequence: vec of (method_name, arguments) for multi-method classes
    Operations(String, Vec<(String, Vec<TypedValue>)>),
}

impl ExampleInput {
    /// Returns true if this is a single-method call.
    #[allow(dead_code)]
    pub fn is_single(&self) -> bool {
        matches!(self, ExampleInput::Single(_))
    }
}

/// Parse the example field, auto-detecting single-method vs operation-sequence format.
pub fn parse_example_input(example: &str) -> anyhow::Result<ExampleInput> {
    // Try operation sequence format: content after "输入：" must start with [" (JSON string array)
    // e.g. 输入：["LRUCache","put","get"]  [[2],[1,1],[2,2]]
    // We check that the input portion begins with [" rather than just checking for " anywhere.
    if let Some(input_start) = example.find("输入：").or_else(|| example.find("输入:")) {
        let after_input = &example[input_start..];
        // Strip the "输入：" or "输入:" prefix
        let content = after_input
            .strip_prefix("输入：")
            .or_else(|| after_input.strip_prefix("输入:"))
            .unwrap_or(after_input)
            .trim();
        // Operation format starts with ["Op1","Op2",...]
        if content.starts_with("[\"") {
            if let Ok(ops) = try_parse_operations(example) {
                return Ok(ops);
            }
        }
        // Chinese-format operation sequence: 输入：capacity = 2，操作：put(1,1), get(1), ...
        if content.contains("操作：") || content.contains("操作:") {
            if let Ok(ops) = try_parse_chinese_ops(example) {
                return Ok(ops);
            }
        }
        // Bare comma-separated method calls (no constructor args):
        // 输入：push(3), push(2), getMin, pop, getMin
        // Detection: no '=' sign, consists of method calls with optional args.
        if !content.contains('=') && !content.contains("操作") {
            // Quick heuristic: first char should be a letter, content looks like method calls
            if content
                .chars()
                .next()
                .map(|c| c.is_ascii_alphabetic())
                .unwrap_or(false)
                && content.contains('(')
            {
                if let Ok(ops) = try_parse_bare_ops(content) {
                    return Ok(ops);
                }
            }
        }
    }
    // Java code block format (208: Trie):
    // "Trie trie = new Trie();\ntrie.insert(\"apple\");\n..."
    // No "输入：" prefix, starts with class instantiation.
    if example.contains("= new ") && example.contains("();") {
        if let Ok(ops) = try_parse_java_block(example) {
            return Ok(ops);
        }
    }
    // Fall back to single-method format
    let params = parse_example(example)?;
    Ok(ExampleInput::Single(params))
}

/// Try to parse the LeetCode operation sequence format:
/// 输入：["LRUCache","put","put","get"]  [[2],[1,1],[2,2],[1]]
fn try_parse_operations(example: &str) -> anyhow::Result<ExampleInput> {
    let content = example
        .lines()
        .find(|line| line.contains("输入：") || line.contains("输入:"))
        .and_then(|line| {
            line.split_once("输入：")
                .or_else(|| line.split_once("输入:"))
                .map(|(_, rest)| rest.trim())
        })
        .context("无法解析操作序列格式")?;

    // Split into two parts: the operation names array and the arguments array
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, ch) in content.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    parts.push(&content[start..=i]);
                    start = content[i + 1..].find('[').map(|p| i + 1 + p).unwrap_or(content.len());
                }
            }
            _ => {}
        }
    }

    // First part: operation names ["Op1","Op2",...]
    let ops_str = parts.first().context("无法解析操作名数组")?;
    // Remove outer brackets and split by comma, respecting quotes
    let ops_inner = &ops_str[1..ops_str.len() - 1];
    let op_names: Vec<String> = ops_inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .collect();

    // Second part: arguments [[arg1],[arg2],...]
    let args_str = parts.get(1).unwrap_or(&"[]");
    let args_inner = &args_str[1..args_str.len() - 1];
    let arg_groups: Vec<String> = smart_split_groups(args_inner);

    let mut operations: Vec<(String, Vec<TypedValue>)> = Vec::new();
    for (i, op_name) in op_names.iter().enumerate() {
        let args = if i < arg_groups.len() {
            let group = &arg_groups[i];
            let group = group.trim();
            if group == "[]" || group.is_empty() {
                vec![]
            } else if group.starts_with('[') && group.ends_with(']') {
                let inner = &group[1..group.len() - 1];
                smart_split(inner, ',')
                    .iter()
                    .map(|s| parse_value(s))
                    .collect::<anyhow::Result<Vec<_>>>()?
            } else {
                vec![]
            }
        } else {
            vec![]
        };
        operations.push((op_name.clone(), args));
    }

    let class_name = operations.first().map(|(n, _)| n.clone()).unwrap_or_default();
    Ok(ExampleInput::Operations(class_name, operations))
}

/// Try to parse Chinese-format operation sequences:
/// 输入：capacity = 2，操作：put(1,1), put(2,2), get(1)
/// Returns ExampleInput::Operations with constructor args as the first operation.
fn try_parse_chinese_ops(example: &str) -> anyhow::Result<ExampleInput> {
    let content = example
        .lines()
        .find(|line| line.contains("输入：") || line.contains("输入:"))
        .and_then(|line| {
            line.split_once("输入：")
                .or_else(|| line.split_once("输入:"))
                .map(|(_, rest)| rest.trim())
        })
        .context("无法解析中文操作格式")?;

    // Split by "操作：" or "操作:"
    let (ctor_part, ops_part) = content
        .split_once("操作：")
        .or_else(|| content.split_once("操作:"))
        .context("未找到'操作：'分隔符")?;

    // Parse constructor arguments from the part before 操作
    let ctor_args = parse_ctor_args(ctor_part)?;

    // Parse operations — only up to the first newline (rest is 输出/分析)
    let ops_text = ops_part.lines().next().unwrap_or(ops_part).trim();
    let method_ops = parse_method_calls(ops_text)?;

    // Build operation list: constructor first, then method calls.
    // The class name string doesn't matter — generate_runner uses analysis.class_name.
    let mut operations = vec![("__ctor__".to_string(), ctor_args)];
    operations.extend(method_ops);

    Ok(ExampleInput::Operations(String::new(), operations))
}

/// Parse constructor arguments like "capacity = 2" or "k = 3, v = 5".
fn parse_ctor_args(s: &str) -> anyhow::Result<Vec<TypedValue>> {
    // Normalize: replace Chinese comma with ASCII comma, then trim trailing commas
    let s = s.trim().replace('，', ",").trim_end_matches(',').trim().to_string();
    if s.is_empty() {
        return Ok(vec![]);
    }
    let mut args = Vec::new();
    for part in smart_split(&s, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(eq_pos) = part.find('=') {
            let value_str = part[eq_pos + 1..].trim();
            args.push(parse_value(value_str)?);
        } else if let Ok(n) = part.parse::<i64>() {
            args.push(TypedValue::Int(n));
        } else {
            args.push(parse_value(part)?);
        }
    }
    Ok(args)
}

/// Parse method call strings like "put(1,1), get(1), put(3,3)".
fn parse_method_calls(s: &str) -> anyhow::Result<Vec<(String, Vec<TypedValue>)>> {
    let s = s.trim();
    let mut ops = Vec::new();
    let mut remaining = s;

    while !remaining.is_empty() {
        // Find next method name (identifier before '(' or before ','/end)
        if let Some(paren_open) = remaining.find('(') {
            // Also check if there's a comma before the paren — if so, the method
            // might be a no-paren method name before the comma.
            let comma_before_paren = remaining[..paren_open].find(',');
            let dot_before_paren = remaining[..paren_open].find('.');
            // Skip if there's a dot before paren (Java method call like obj.method())
            // — but we don't allow dots in our format.
            if let Some(_dot_pos) = dot_before_paren {
                // This is an object.method() call — not our target format
                break;
            }
            // If there's a comma between the method name candidate and '(',
            // the '(' might belong to a later call, not this method.
            // In our format, method(args) always has '(' right after the name.
            let method_name = if let Some(comma_pos) = comma_before_paren {
                // Comma before paren: treat everything before comma as a no-paren method
                let name = remaining[..comma_pos].trim().to_string();
                // Advance remaining past this method name
                remaining = remaining[comma_pos + 1..].trim_start();
                if name.is_empty()
                    || !name.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    break;
                }
                ops.push((name, vec![]));
                continue;
            } else {
                remaining[..paren_open].trim().to_string()
            };

            // Validate method name: must be a valid Java identifier
            if method_name.is_empty()
                || !method_name
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_')
            {
                break;
            }

            // Find matching closing paren (respects nesting)
            let after_open = &remaining[paren_open + 1..];
            let mut depth: i32 = 1;
            let mut paren_close: Option<usize> = None;
            for (i, ch) in after_open.char_indices() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            paren_close = Some(i);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            let paren_close = paren_close.context("未找到匹配的右括号")?;
            let args_str = &after_open[..paren_close];
            let args = if args_str.trim().is_empty() {
                vec![]
            } else {
                smart_split(args_str, ',')
                    .iter()
                    .map(|s| parse_value(s))
                    .collect::<anyhow::Result<Vec<_>>>()?
            };

            ops.push((method_name, args));

            // Advance past this method call: "method(args)"
            let call_end = paren_open + 1 + paren_close + 1;
            remaining = remaining[call_end..].trim_start();
            // Skip comma separator (ASCII or Chinese)
            if remaining.starts_with(',') {
                remaining = remaining[1..].trim_start();
            } else if remaining.starts_with('，') {
                remaining = remaining['，'.len_utf8()..].trim_start();
            }
        } else {
            // No '(' at all — remaining is comma-separated no-paren method names
            let names: Vec<String> = remaining
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            for name in &names {
                if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    break;
                }
                ops.push((name.clone(), vec![]));
            }
            break;
        }
    }

    Ok(ops)
}

/// Try to parse bare comma-separated method calls:
/// 输入：push(3), push(2), getMin, pop, getMin
/// No constructor args, no "操作：" separator.
fn try_parse_bare_ops(content: &str) -> anyhow::Result<ExampleInput> {
    // Take only the first line (before \n输出： etc.)
    let ops_text = content.lines().next().unwrap_or(content).trim();
    let mut method_ops = parse_method_calls(ops_text)?;

    if method_ops.is_empty() {
        anyhow::bail!("无法解析操作方法调用");
    }

    // Insert a dummy constructor at position 0
    method_ops.insert(0, ("__ctor__".to_string(), vec![]));
    Ok(ExampleInput::Operations(String::new(), method_ops))
}

/// Try to parse Java code block format (208: Trie):
/// Trie trie = new Trie();
/// trie.insert("apple");
/// trie.search("apple");   // true
fn try_parse_java_block(example: &str) -> anyhow::Result<ExampleInput> {
    // Parse the first line: "Trie trie = new Trie();"
    let first_line = example.lines().next().context("无法解析 Java 代码块格式")?;
    let first_line = first_line.trim();

    // Extract constructor args from "new ClassName(args)"
    let ctor_args = if let Some(new_start) = first_line.find("new ") {
        let after_new = &first_line[new_start + 4..]; // skip "new "
        if let Some(paren_open) = after_new.find('(') {
            if let Some(paren_close) = after_new.find(')') {
                let args_str = after_new[paren_open + 1..paren_close].trim();
                if args_str.is_empty() {
                    vec![]
                } else {
                    smart_split(args_str, ',')
                        .iter()
                        .map(|s| parse_value(s))
                        .collect::<anyhow::Result<Vec<_>>>()?
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Parse subsequent method calls
    let mut operations = vec![("__ctor__".to_string(), ctor_args)];

    for line in example.lines().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("输出") || trimmed.starts_with("分析") {
            continue;
        }

        // Strip inline comments: "// true" or "// anything"
        let trimmed = if let Some(comment_pos) = trimmed.find("//") {
            trimmed[..comment_pos].trim()
        } else {
            trimmed
        };

        if trimmed.is_empty() {
            continue;
        }

        // Parse "obj.method(args);" or "obj.method();"
        if let Some(dot_pos) = trimmed.find('.') {
            let after_dot = &trimmed[dot_pos + 1..];
            if let Some(paren_open) = after_dot.find('(') {
                let method_name = after_dot[..paren_open].trim().to_string();
                if let Some(paren_close) = after_dot.rfind(')') {
                    let args_str = after_dot[paren_open + 1..paren_close].trim();
                    let args = if args_str.is_empty() {
                        vec![]
                    } else {
                        smart_split(args_str, ',')
                            .iter()
                            .map(|s| parse_value(s))
                            .collect::<anyhow::Result<Vec<_>>>()?
                    };
                    operations.push((method_name, args));
                }
            }
        }
    }

    if operations.len() <= 1 {
        anyhow::bail!("无法解析 Java 代码块中的方法调用");
    }

    Ok(ExampleInput::Operations(String::new(), operations))
}

/// Split by commas at the top level, respecting nested brackets.
fn smart_split_groups(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for ch in s.chars() {
        match ch {
            '[' => {
                depth += 1;
                current.push(ch);
            }
            ']' => {
                depth -= 1;
                current.push(ch);
                if depth == 0 {
                    result.push(current.trim().to_string());
                    current = String::new();
                }
            }
            ',' if depth == 0 => {
                // separator between groups, skip
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

/// Parse the example field into a list of (param_name, value) pairs (single-method format).
pub fn parse_example(example: &str) -> anyhow::Result<Vec<(String, TypedValue)>> {
    // Find the first "输入：" line
    let input_line = example
        .lines()
        .find(|line| line.contains("输入：") || line.contains("输入:"))
        .context("无法解析示例格式：未找到'输入：'")?;

    // Extract content after "输入：" or "输入:"
    let content = input_line
        .split_once("输入：")
        .or_else(|| input_line.split_once("输入:"))
        .map(|(_, rest)| rest.trim())
        .context("无法解析示例格式")?;

    // Remove trailing "输出：" part if on same line
    let content = if let Some(pos) = content.find("输出：") {
        &content[..pos]
    } else if let Some(pos) = content.find("输出:") {
        &content[..pos]
    } else if let Some(pos) = content.find('→') {
        &content[..pos]
    } else if let Some(pos) = content.find("=>") {
        &content[..pos]
    } else {
        content
    }
    .trim()
    .trim_end_matches("→")
    .trim();

    // Split by ", " then by " = " to get name-value pairs
    let mut params = Vec::new();
    let pairs = smart_split(content, ',');
    for pair in pairs {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some(eq_pos) = pair.find('=') {
            let name = pair[..eq_pos].trim().to_string();
            let value_str = pair[eq_pos + 1..].trim();
            let value = parse_value(value_str)?;
            params.push((name, value));
        }
    }

    if params.is_empty() {
        anyhow::bail!("无法从示例中解析出参数")
    }

    Ok(params)
}

/// Smart split that respects brackets and quotes.
fn smart_split(s: &str, delimiter: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth_bracket = 0;
    let mut in_quote = false;

    for ch in s.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                current.push(ch);
            }
            '[' => {
                depth_bracket += 1;
                current.push(ch);
            }
            ']' => {
                depth_bracket -= 1;
                current.push(ch);
            }
            c if c == delimiter && depth_bracket == 0 && !in_quote => {
                result.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }
    result
}

/// Parse a single value string into a TypedValue.
fn parse_value(s: &str) -> anyhow::Result<TypedValue> {
    let s = s.trim();

    // String literal
    if s.starts_with('"') && s.ends_with('"') {
        return Ok(TypedValue::String(s[1..s.len() - 1].to_string()));
    }

    // Boolean
    if s == "true" {
        return Ok(TypedValue::Bool(true));
    }
    if s == "false" {
        return Ok(TypedValue::Bool(false));
    }

    // null
    if s == "null" {
        return Ok(TypedValue::Int(0)); // Treat null as 0 for now
    }

    // Array
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        if inner.trim().is_empty() {
            return Ok(TypedValue::Array(vec![]));
        }

        // Check if nested array
        if inner.trim().starts_with('[') {
            // Nested array like [[1,2],[3,4]]
            let rows = parse_nested_array(s)?;
            return Ok(TypedValue::NestedArray(rows));
        }

        // Check for null values (TreeNode array)
        if inner.contains("null") {
            let elems = parse_tree_node_array(inner)?;
            return Ok(TypedValue::TreeNodeArray(elems));
        }

        // Simple array
        let elems = smart_split(inner, ',')
            .iter()
            .map(|e| parse_value(e))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(TypedValue::Array(elems));
    }

    // Linked-list notation: 1->2->3->4 or 1->2->3->null
    // May have trailing annotations like （从 3 开始相交） — strip them.
    if s.contains("->") && !s.starts_with('[') && !s.starts_with('{') {
        // Strip trailing Chinese annotations: （...） 或 (...)
        let cleaned = strip_trailing_annotation(s);
        let elems: Vec<TypedValue> = cleaned
            .split("->")
            .map(|part| {
                let part = part.trim();
                if part == "null" {
                    Ok(TypedValue::Int(0)) // sentinel
                } else if let Ok(n) = part.parse::<i64>() {
                    Ok(TypedValue::Int(n))
                } else {
                    // strip trailing non-digit characters (e.g., "4）" → "4")
                    let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() {
                        Ok(TypedValue::Int(digits.parse().unwrap_or(0)))
                    } else {
                        Ok(TypedValue::String(part.to_string()))
                    }
                }
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        // Only treat as linked list if we got mostly numbers
        let num_count = elems.iter().filter(|e| matches!(e, TypedValue::Int(_))).count();
        if num_count as f64 / elems.len() as f64 > 0.5 {
            return Ok(TypedValue::Array(elems));
        }
        // Otherwise, treat as regular string with "->" in it
    }

    // Integer
    if let Ok(n) = s.parse::<i64>() {
        return Ok(TypedValue::Int(n));
    }

    // Fallback: treat as string
    Ok(TypedValue::String(s.to_string()))
}

/// Parse a nested array like [[1,2],[3,4]] or [[1,4,5],[1,3,4],[2,6]].
/// Walks the inner string character-by-character, respecting bracket depth,
/// to correctly split sub-arrays even when they contain commas.
fn parse_nested_array(s: &str) -> anyhow::Result<Vec<Vec<TypedValue>>> {
    let s = s.trim();
    if !s.starts_with("[[") || !s.ends_with("]]") {
        anyhow::bail!("无法解析嵌套数组: {}", s)
    }

    // Strip only the OUTER [ and ] (1 char each), preserving inner [[row1],[row2]] brackets.
    let inner = &s[1..s.len() - 1];
    let mut rows: Vec<Vec<TypedValue>> = Vec::new();
    let mut depth = 0;
    let mut current = String::new();

    for ch in inner.chars() {
        match ch {
            '[' => {
                depth += 1;
                current.push(ch);
            }
            ']' => {
                depth -= 1;
                current.push(ch);
                // When depth returns to 0, we've closed a sub-array
                if depth == 0 {
                    let row_str = current.trim().to_string();
                    if !row_str.is_empty() {
                        // Parse the row as an array
                        let row = if row_str.starts_with('[') && row_str.ends_with(']') {
                            let row_inner = &row_str[1..row_str.len() - 1];
                            if row_inner.trim().is_empty() {
                                vec![]
                            } else {
                                smart_split(row_inner, ',')
                                    .iter()
                                    .map(|e| parse_value(e))
                                    .collect::<anyhow::Result<Vec<_>>>()?
                            }
                        } else {
                            vec![parse_value(&row_str)?]
                        };
                        rows.push(row);
                    }
                    current = String::new();
                }
            }
            ',' if depth == 0 => {
                // separator between sub-arrays — skip
            }
            _ => {
                current.push(ch);
            }
        }
    }

    Ok(rows)
}

/// Parse a TreeNode array with possible null values.
fn parse_tree_node_array(inner: &str) -> anyhow::Result<Vec<Option<i64>>> {
    smart_split(inner, ',')
        .iter()
        .map(|s| {
            let s = s.trim();
            if s == "null" {
                Ok(None)
            } else if let Ok(n) = s.parse::<i64>() {
                Ok(Some(n))
            } else {
                // Could be a string; try to parse anyway
                anyhow::bail!("无法解析树节点值: {}", s)
            }
        })
        .collect()
}

/// Strip trailing annotation text like "（从 3 开始相交）" or "(from node 3)"
/// from a value string. Used to clean linked-list notation values.
fn strip_trailing_annotation(s: &str) -> String {
    let s = s.trim();
    // Chinese parentheses
    if let Some(pos) = s.find('（') {
        return s[..pos].trim().to_string();
    }
    // ASCII parentheses used as annotation (not part of method signature)
    if let Some(pos) = s.find("(从") {
        return s[..pos].trim().to_string();
    }
    if let Some(pos) = s.find("(from") {
        return s[..pos].trim().to_string();
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_two_sum() {
        let example = "输入：nums = [2,7,11,15], target = 9\n输出：[0,1]";
        let params = parse_example(example).unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].0, "nums");
        assert_eq!(params[1].0, "target");
        match &params[0].1 {
            TypedValue::Array(arr) => assert_eq!(arr.len(), 4),
            _ => panic!("Expected array"),
        }
        match &params[1].1 {
            TypedValue::Int(n) => assert_eq!(*n, 9),
            _ => panic!("Expected int"),
        }
    }

    #[test]
    fn test_parse_string_input() {
        let example = "输入：s = \"abcabcbb\"\n输出：3";
        let params = parse_example(example).unwrap();
        assert_eq!(params.len(), 1);
        match &params[0].1 {
            TypedValue::String(s) => assert_eq!(s, "abcabcbb"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_binary_search() {
        let example =
            "输入：nums = [1,3,5,6], target = 5   → 输出：2\n输入：nums = [1,3,5,6], target = 2   → 输出：1";
        let params = parse_example(example).unwrap();
        assert_eq!(params.len(), 2);
        match &params[1].1 {
            TypedValue::Int(n) => assert_eq!(*n, 5),
            _ => panic!("Expected int"),
        }
    }

    #[test]
    fn test_parse_chinese_ops() {
        let example = "输入：capacity = 2，操作：put(1,1), put(2,2), get(1), put(3,3), get(2)\n输出：get(1)=1, get(2)=-1（2 被淘汰）";
        let input = parse_example_input(example).unwrap();
        match &input {
            ExampleInput::Operations(_class_name, ops) => {
                // 6 operations: __ctor__ + put + put + get + put + get
                assert_eq!(ops.len(), 6, "expected 6 ops, got {}", ops.len());
                // ops[0]: Constructor (capacity=2)
                assert_eq!(ops[0].0, "__ctor__");
                assert_eq!(ops[0].1.len(), 1);
                assert!(matches!(&ops[0].1[0], TypedValue::Int(2)));
                // ops[1]: put(1,1)
                assert_eq!(ops[1].0, "put");
                assert_eq!(ops[1].1.len(), 2);
                // ops[2]: put(2,2)
                assert_eq!(ops[2].0, "put");
                // ops[3]: get(1)
                assert_eq!(ops[3].0, "get");
                assert_eq!(ops[3].1.len(), 1);
                assert!(matches!(&ops[3].1[0], TypedValue::Int(1)));
                // ops[4]: put(3,3)
                assert_eq!(ops[4].0, "put");
                // ops[5]: get(2)
                assert_eq!(ops[5].0, "get");
                assert!(matches!(&ops[5].1[0], TypedValue::Int(2)));
            }
            other => panic!("Expected Operations, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_chinese_ops_no_ctor_args() {
        // Some classes like Trie() have no constructor args
        let example = "输入：操作：insert(\"apple\"), search(\"apple\"), startsWith(\"app\")\n输出：null, true, true";
        let input = parse_example_input(example).unwrap();
        match &input {
            ExampleInput::Operations(_class_name, ops) => {
                assert_eq!(ops[0].0, "__ctor__");
                assert!(ops[0].1.is_empty(), "expected no ctor args");
                assert_eq!(ops[1].0, "insert");
                assert_eq!(ops[2].0, "search");
                assert_eq!(ops[3].0, "startsWith");
            }
            other => panic!("Expected Operations, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_bare_ops_minstack() {
        let example = "输入：push(3), push(2), getMin, pop, getMin\n输出：2, 3";
        let input = parse_example_input(example).unwrap();
        match &input {
            ExampleInput::Operations(_cn, ops) => {
                assert_eq!(ops.len(), 6, "expected 6 ops, got {}", ops.len());
                // __ctor__ with no args
                assert_eq!(ops[0].0, "__ctor__");
                assert!(ops[0].1.is_empty());
                // push(3)
                assert_eq!(ops[1].0, "push");
                assert_eq!(ops[1].1.len(), 1);
                // push(2)
                assert_eq!(ops[2].0, "push");
                // getMin (no args)
                assert_eq!(ops[3].0, "getMin");
                assert!(ops[3].1.is_empty());
                // pop (no args)
                assert_eq!(ops[4].0, "pop");
                assert!(ops[4].1.is_empty());
                // getMin again
                assert_eq!(ops[5].0, "getMin");
            }
            other => panic!("Expected Operations, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_bare_ops_median_finder() {
        let example = "输入：addNum(1), addNum(2), findMedian, addNum(3), findMedian";
        let input = parse_example_input(example).unwrap();
        match &input {
            ExampleInput::Operations(_cn, ops) => {
                assert_eq!(ops.len(), 6);
                assert_eq!(ops[0].0, "__ctor__");
                assert_eq!(ops[1].0, "addNum");
                assert_eq!(ops[2].0, "addNum");
                assert_eq!(ops[3].0, "findMedian");
                assert_eq!(ops[4].0, "addNum");
                assert_eq!(ops[5].0, "findMedian");
            }
            other => panic!("Expected Operations, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_java_block_trie() {
        let example = "Trie trie = new Trie();\ntrie.insert(\"apple\");\ntrie.search(\"apple\");   // true\ntrie.startsWith(\"app\"); // true";
        let input = parse_example_input(example).unwrap();
        match &input {
            ExampleInput::Operations(_cn, ops) => {
                assert_eq!(ops.len(), 4, "expected 4 ops, got {}", ops.len());
                // Constructor
                assert_eq!(ops[0].0, "__ctor__");
                assert!(ops[0].1.is_empty());
                // insert("apple")
                assert_eq!(ops[1].0, "insert");
                assert_eq!(ops[1].1.len(), 1);
                match &ops[1].1[0] {
                    TypedValue::String(s) => assert_eq!(s, "apple"),
                    other => panic!("Expected String, got {:?}", other),
                }
                // search("apple")
                assert_eq!(ops[2].0, "search");
                // startsWith("app")
                assert_eq!(ops[3].0, "startsWith");
            }
            other => panic!("Expected Operations, got {:?}", other),
        }
    }

    #[test]
    fn test_java_literal() {
        let arr = TypedValue::Array(vec![
            TypedValue::Int(2),
            TypedValue::Int(7),
            TypedValue::Int(11),
            TypedValue::Int(15),
        ]);
        assert_eq!(arr.to_java_literal(), "{2, 7, 11, 15}");
        assert_eq!(arr.java_type(), "int[]");

        let s = TypedValue::String("abc".into());
        assert_eq!(s.to_java_literal(), "\"abc\"");
    }
}
