/// Analysis result for a Java answer code string.
#[derive(Debug, Clone)]
pub struct Analysis {
    /// The class name, e.g. "Solution"
    pub class_name: String,
    /// All public method signatures
    pub public_methods: Vec<MethodInfo>,
    /// Local variable declarations in the primary method, with scope info
    pub var_decls: Vec<VarDecl>,
    /// Set of custom types needed (ListNode, TreeNode, Node)
    pub needs_types: Vec<String>,
    /// The full answer code (for reference)
    #[allow(dead_code)]
    pub answer: String,
    /// Line-number → code-text mapping for the answer
    pub code_lines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub return_type: String,
    pub name: String,
    pub params: Vec<(String, String)>, // (type, name)
    pub body_start_line: usize,        // 1-indexed
    pub body_end_line: usize,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    #[allow(dead_code)]
    pub var_type: String,
    pub line: usize,       // 1-indexed declaration line
    pub scope_end: usize,  // 1-indexed line where variable goes out of scope
}

/// Analyze a Java answer string and extract metadata.
pub fn analyze(answer: &str) -> Result<Analysis, String> {
    let code_lines: Vec<String> = answer.lines().map(|s| s.to_string()).collect();

    // Extract class name
    let class_name = extract_class_name(answer)?;

    // Extract public methods
    let public_methods = extract_methods(answer)?;

    // Detect custom type dependencies
    let needs_types = detect_custom_types(answer);

    // For the primary (first public) method, extract variable declarations
    let var_decls = if let Some(primary) = public_methods.first() {
        extract_vars(&code_lines, primary)?
    } else {
        vec![]
    };

    Ok(Analysis {
        class_name,
        public_methods,
        var_decls,
        needs_types,
        answer: answer.to_string(),
        code_lines,
    })
}

fn extract_class_name(code: &str) -> Result<String, String> {
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("class ") || trimmed.starts_with("public class ") {
            let name = trimmed
                .trim_start_matches("public ")
                .trim_start_matches("class ")
                .split(|c: char| c == '{' || c.is_whitespace())
                .next()
                .unwrap_or("Unknown");
            return Ok(name.to_string());
        }
    }
    Err("无法找到类定义".into())
}

fn extract_methods(code: &str) -> Result<Vec<MethodInfo>, String> {
    let mut methods = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        // Match public method declarations
        if trimmed.starts_with("public ") && trimmed.contains('(') && trimmed.contains(')') {
            // Check if it's a method (not a constructor with class name)
            let has_return_type = !trimmed.contains(" class ")
                && (trimmed.contains(" void ")
                    || trimmed.contains(" int ")
                    || trimmed.contains(" boolean ")
                    || trimmed.contains(" String ")
                    || trimmed.contains(" List")
                    || trimmed.contains(" Map")
                    || trimmed.contains(" double ")
                    || trimmed.contains(" char")
                    || trimmed.contains(" ListNode")
                    || trimmed.contains(" TreeNode"));

            if has_return_type || trimmed.contains(" public ") {
                // Parse the signature
                let (return_type, name, params, body_start) = parse_method_sig(&lines, i)?;
                let body_end = find_method_end(&lines, body_start);

                methods.push(MethodInfo {
                    return_type,
                    name,
                    params,
                    body_start_line: body_start + 1, // 1-indexed
                    body_end_line: body_end + 1,
                });

                i = body_end + 1;
                continue;
            }
        }
        i += 1;
    }

    if methods.is_empty() {
        return Err("无法找到 public 方法".into());
    }

    Ok(methods)
}

fn parse_method_sig(
    lines: &[&str],
    start: usize,
) -> Result<(String, String, Vec<(String, String)>, usize), String> {
    // Build the full signature (may span multiple lines)
    let mut sig = String::new();
    let mut i = start;
    let mut body_line = start;
    while i < lines.len() {
        sig.push_str(lines[i].trim());
        sig.push(' ');
        if lines[i].contains('{') {
            body_line = i;
            break;
        }
        i += 1;
    }

    // Find the opening paren
    let paren_open = sig.find('(').ok_or("方法签名缺少 '('")?;
    let paren_close = sig.rfind(')').ok_or("方法签名缺少 ')'")?;

    // Everything before the paren is "return_type method_name"
    let before_paren = sig[..paren_open].trim();
    let parts: Vec<&str> = before_paren.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("无法解析方法签名".into());
    }

    // Last word before paren is method name
    let method_name = parts.last().unwrap().to_string();
    // Everything before the method name is the return type (strip modifiers)
    let mut return_parts: Vec<&str> = Vec::new();
    for part in &parts[..parts.len() - 1] {
        let s = *part;
        if s != "public" && s != "protected" && s != "private" && s != "static" {
            return_parts.push(s);
        }
    }
    let return_type = return_parts.join(" ");

    // Parse parameters
    let params_str = sig[paren_open + 1..paren_close].trim();
    let params = if params_str.is_empty() {
        vec![]
    } else {
        params_str
            .split(',')
            .filter_map(|p| {
                let p = p.trim();
                if p.is_empty() {
                    return None;
                }
                let parts: Vec<&str> = p.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ptype = parts[..parts.len() - 1].join(" ");
                    let pname = parts.last().unwrap().to_string();
                    Some((ptype, pname))
                } else {
                    None
                }
            })
            .collect()
    };

    Ok((return_type, method_name, params, body_line))
}

fn find_method_end(lines: &[&str], body_start: usize) -> usize {
    let mut depth = 0;
    let mut found_open = false;
    for (i, line) in lines.iter().enumerate().skip(body_start) {
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    found_open = true;
                }
                '}' => {
                    depth -= 1;
                    if found_open && depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
    }
    lines.len() - 1
}

fn detect_custom_types(code: &str) -> Vec<String> {
    let mut types = Vec::new();
    if code.contains("ListNode") {
        types.push("ListNode".to_string());
    }
    if code.contains("TreeNode") {
        types.push("TreeNode".to_string());
    }
    if code.contains("class Node") || code.contains("Node ") {
        types.push("Node".to_string());
    }
    types
}

/// Extract variable declarations with scope tracking within a method body.
#[allow(unused_assignments, unused_variables)]
fn extract_vars(code_lines: &[String], method: &MethodInfo) -> Result<Vec<VarDecl>, String> {
    let mut vars = Vec::new();
    let start = method.body_start_line.saturating_sub(1); // 0-indexed
    let end = method.body_end_line.saturating_sub(1);

    // Track brace depth
    let mut depth = 0;
    let mut scope_stack: Vec<usize> = Vec::new(); // line numbers of scope-open braces

    for i in start..=end.min(code_lines.len() - 1) {
        let line = &code_lines[i];
        let trimmed = line.trim();

        // Count braces
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    scope_stack.push(i + 1); // 1-indexed line
                }
                '}' => {
                    depth -= 1;
                    scope_stack.pop();
                }
                _ => {}
            }
        }

        // Skip comment-only lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // Detect variable declarations
        if let Some(var) = try_parse_declaration(trimmed, i + 1) {
            // Scope ends at the line where depth drops below current
            let scope_end = end + 1;
            vars.push(VarDecl {
                name: var.0,
                var_type: var.1,
                line: i + 1, // 1-indexed
                scope_end,
            });
        }
    }

    // Recalculate scope ends
    recalc_scopes(&mut vars, code_lines, start, end);

    Ok(vars)
}

fn try_parse_declaration(line: &str, _line_num: usize) -> Option<(String, String)> {
    let trimmed = line.trim();

    // Skip pure control flow lines (but NOT for/while that may contain declarations)
    if trimmed.starts_with("if ")
        || trimmed.starts_with("return ")
        || trimmed.starts_with("} else")
        || trimmed.starts_with("try ")
        || trimmed.starts_with("catch")
    {
        return None;
    }

    // Must contain '=' (assignment/declaration) or ':' (for-each loop)
    let has_eq = trimmed.contains('=');
    let has_colon = trimmed.contains(':') && (trimmed.starts_with("for (") || trimmed.starts_with("for("));

    if !has_eq && !has_colon {
        return None;
    }

    // Must end with ';' or '{' (for loop headers with declarations)
    if !trimmed.ends_with(';') && !trimmed.ends_with('{') {
        return None;
    }

    if has_colon {
        // Handle for-each: for (Type name : collection)
        // Extract the part between '(' and ':'
        if let Some(paren_open) = trimmed.find('(') {
            if let Some(colon_pos) = trimmed.find(':') {
                let between = trimmed[paren_open + 1..colon_pos].trim();
                let parts: Vec<&str> = between.split_whitespace().collect();
                if parts.len() >= 2 {
                    let type_str = parts[..parts.len() - 1].join(" ");
                    let var_name = parts.last().unwrap().to_string();
                    let has_type = type_str.contains("int")
                        || type_str.contains("char")
                        || type_str.contains("boolean")
                        || type_str.contains("String")
                        || type_str.contains("double")
                        || type_str.contains("long")
                        || type_str.contains("ListNode")
                        || type_str.contains("TreeNode");
                    if has_type {
                        return Some((var_name, type_str));
                    }
                }
            }
        }
        return None;
    }

    // Must have a type keyword before the variable name
    let eq_pos = trimmed.find('=')?;
    let before_eq = trimmed[..eq_pos].trim();
    let words: Vec<&str> = before_eq.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let var_name = words.last().unwrap();
    let type_words = &words[..words.len() - 1];

    // Check if there's actually a type declaration (not just a reassignment)
    if type_words.is_empty() {
        // Reassignment like "i = 0" or "map.put(...)", not a declaration
        return None;
    }

    // Valid type indicators
    let type_str = type_words.join(" ");
    let has_type = type_str.contains("int")
        || type_str.contains("char")
        || type_str.contains("boolean")
        || type_str.contains("String")
        || type_str.contains("Map")
        || type_str.contains("List")
        || type_str.contains("Set")
        || type_str.contains("Deque")
        || type_str.contains("TreeNode")
        || type_str.contains("ListNode")
        || type_str.contains("double")
        || type_str.contains("long");

    if has_type {
        // Make sure var_name is actually a valid identifier
        let name = var_name.trim_end_matches(';').trim_end_matches('{');
        if name.chars().all(|c| c.is_alphanumeric() || c == '_') && !name.is_empty() {
            return Some((name.to_string(), type_str));
        }
    }

    None
}

fn recalc_scopes(vars: &mut [VarDecl], code_lines: &[String], body_start: usize, body_end: usize) {
    for var in vars.iter_mut() {
        let mut depth = 0;
        let decl_line = var.line.saturating_sub(1);
        let mut scope_end = body_end + 1;

        // Count depth at declaration point
        for i in body_start..=decl_line.min(code_lines.len() - 1) {
            for ch in code_lines[i].chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
        }

        let decl_depth = depth;

        // Scan forward until depth drops below decl_depth
        for i in (decl_line + 1)..=body_end.min(code_lines.len() - 1) {
            for ch in code_lines[i].chars() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        if depth == decl_depth {
                            scope_end = i + 1; // 1-indexed
                        }
                        depth -= 1;
                    }
                    _ => {}
                }
            }
            if scope_end != body_end + 1 {
                break;
            }
        }

        var.scope_end = scope_end;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_SUM: &str = "import java.util.*;\n\nclass Solution {\n    public int[] twoSum(int[] nums, int target) {\n        Map<Integer, Integer> map = new HashMap<>();\n        for (int i = 0; i < nums.length; i++) {\n            int complement = target - nums[i];\n            if (map.containsKey(complement)) {\n                return new int[]{map.get(complement), i};\n            }\n            map.put(nums[i], i);\n        }\n        return new int[]{};\n    }\n}";

    #[test]
    fn test_extract_class_name() {
        assert_eq!(extract_class_name(TWO_SUM).unwrap(), "Solution");
    }

    #[test]
    fn test_extract_methods() {
        let methods = extract_methods(TWO_SUM).unwrap();
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].name, "twoSum");
        assert_eq!(methods[0].return_type, "int[]");
        assert_eq!(methods[0].params.len(), 2);
        assert_eq!(methods[0].params[0].0, "int[]");
        assert_eq!(methods[0].params[0].1, "nums");
        assert_eq!(methods[0].params[1].1, "target");
    }

    #[test]
    fn test_analyze() {
        let analysis = analyze(TWO_SUM).unwrap();
        assert_eq!(analysis.class_name, "Solution");
        assert_eq!(analysis.public_methods.len(), 1);
        assert!(!analysis.var_decls.is_empty());
        // Should find: map, i, complement
        let var_names: Vec<&str> = analysis.var_decls.iter().map(|v| v.name.as_str()).collect();
        assert!(var_names.contains(&"map"));
        assert!(var_names.contains(&"i"));
        assert!(var_names.contains(&"complement"));
    }
}
