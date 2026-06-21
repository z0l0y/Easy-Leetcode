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

    /// Get the main method return type if this is the expected output.
    #[allow(unused_variables)]
    pub fn to_java_init(&self, var_name: &str) -> String {
        match self {
            TypedValue::Int(n) => format!("int {} = {};", var_name, n),
            TypedValue::String(s) => format!("String {} = \"{}\";", var_name, s),
            TypedValue::Bool(b) => format!("boolean {} = {};", var_name, b),
            TypedValue::Array(elems) => {
                let lit = self.to_java_literal();
                format!("int[] {} = {};", var_name, lit)
            }
            TypedValue::NestedArray(rows) => {
                let lit = self.to_java_literal();
                format!("int[][] {} = {};", var_name, lit)
            }
            TypedValue::TreeNodeArray(elems) => {
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
    // Try operation sequence format first: ["Op1","Op2"] \n [[args1],[args2]]
    if example.contains('[') && example.contains('"') && example.contains("],[") {
        if let Ok(ops) = try_parse_operations(example) {
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

    // Integer
    if let Ok(n) = s.parse::<i64>() {
        return Ok(TypedValue::Int(n));
    }

    // Fallback: treat as string
    Ok(TypedValue::String(s.to_string()))
}

/// Parse a nested array like [[1,2],[3,4]].
fn parse_nested_array(s: &str) -> anyhow::Result<Vec<Vec<TypedValue>>> {
    let s = s.trim();
    if !s.starts_with("[[") || !s.ends_with("]]") {
        anyhow::bail!("无法解析嵌套数组: {}", s)
    }

    let inner = &s[2..s.len() - 2];
    let rows = smart_split(inner, ']')
        .iter()
        .map(|row| {
            let row = row.trim().trim_start_matches(',').trim();
            let row = row.trim_start_matches('[');
            if row.is_empty() {
                return Ok(vec![]);
            }
            smart_split(row, ',')
                .iter()
                .map(|e| parse_value(e))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
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
