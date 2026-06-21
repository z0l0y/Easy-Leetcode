use super::analyzer::Analysis;
use super::example::TypedValue;

/// Generate a complete, compilable TraceRunner.java file.
pub fn generate(analysis: &Analysis, params: &[(String, TypedValue)]) -> Result<String, String> {
    let mut out = String::new();

    // 0. Import statement (must be first)
    out.push_str("import java.util.*;\n\n");

    // 1. Inject custom type definitions (ListNode, TreeNode)
    for ty in &analysis.needs_types {
        out.push_str(&generate_type_def(ty));
        out.push('\n');
    }

    // 2. The Solution class with instrumentation
    if let Some(primary) = analysis.public_methods.first() {
        out.push_str(&generate_instrumented_solution(
            analysis,
            primary,
            &analysis.var_decls,
        )?);
    } else {
        return Err("没有找到 public 方法".into());
    }

    // 3. The TraceRunner class with main()
    out.push('\n');
    out.push_str(&generate_runner(analysis, params)?);

    Ok(out)
}

fn generate_type_def(type_name: &str) -> String {
    match type_name {
        "ListNode" => r#"class ListNode {
    int val;
    ListNode next;
    ListNode(int x) { val = x; }
    public String toString() {
        StringBuilder sb = new StringBuilder();
        ListNode cur = this;
        while (cur != null) {
            sb.append(cur.val);
            if (cur.next != null) sb.append("->");
            cur = cur.next;
        }
        return sb.toString();
    }
}"#
        .to_string(),

        "TreeNode" => r#"class TreeNode {
    int val;
    TreeNode left;
    TreeNode right;
    TreeNode(int x) { val = x; }
    public String toString() { return String.valueOf(val); }
}"#
        .to_string(),

        "Node" => r#"class Node {
    int val;
    Node next;
    Node random;
    Node(int x) { val = x; }
    public String toString() { return String.valueOf(val); }
}"#
        .to_string(),

        _ => String::new(),
    }
}

fn generate_instrumented_solution(
    analysis: &Analysis,
    method: &super::analyzer::MethodInfo,
    var_decls: &[super::analyzer::VarDecl],
) -> Result<String, String> {
    let mut out = String::new();

    // --- Generate instrumented method ---
    // Write lines from the original code (0 to body_start), skipping imports
    let code_lines = &analysis.code_lines;
    let body_start = method.body_start_line.saturating_sub(1); // 0-indexed
    let body_end = method.body_end_line.saturating_sub(1);

    // Write lines before method body (class header, method signature, opening brace)
    // Skip import and package lines from original code (already added at top)
    for i in 0..=body_start {
        let line = &code_lines[i];
        let trimmed = line.trim();
        if trimmed.starts_with("import ") || trimmed.starts_with("package ") {
            continue;
        }
        out.push_str(line);
        out.push('\n');

        // If this is the opening brace of the method, insert initial capture
        if i == body_start {
            // Get parameter names
            let param_names: Vec<String> = method
                .params
                .iter()
                .map(|(_, name)| format!("\"{}\"", name))
                .collect();
            let param_values: Vec<String> =
                method.params.iter().map(|(_, name)| name.clone()).collect();

            if !param_names.is_empty() {
                out.push_str(&format!(
                    "        __t({}, new String[]{{{}}}, {});\n",
                    i + 1,
                    param_names.join(", "),
                    param_values.join(", ")
                ));
            }
        }
    }

    // Write instrumented body lines — capture state at EVERY meaningful line
    for i in (body_start + 1)..=body_end.min(code_lines.len() - 1) {
        let line = &code_lines[i];
        let trimmed = line.trim();

        // Skip blank/comment-only lines (don't instrument)
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            out.push_str(line);
            out.push('\n');
            continue;
        }

        // ── Statement type detection ──────────────────────────────

        // return statements: capture BEFORE we leave
        let is_return = trimmed.starts_with("return ") || trimmed == "return;";

        // Any statement ending with ; (assignments, method calls, declarations)
        let is_statement = trimmed.ends_with(';')
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("return ");

        // if-condition headers: if (...) {  or  if (...) \n  (single-line if without braces)
        let is_if_header = (trimmed.starts_with("if (") || trimmed.starts_with("if("))
            && (trimmed.ends_with('{') || trimmed.ends_with(')'));

        // else / else-if transitions
        let is_else_header = (trimmed.starts_with("} else") || trimmed.starts_with("}else"))
            && (trimmed.ends_with('{') || trimmed.contains("if ("));

        // Loop headers: for (...) {  or  while (...) {
        let is_loop_header = (trimmed.starts_with("for (") || trimmed.starts_with("for(")
            || trimmed.starts_with("while (") || trimmed.starts_with("while("))
            && trimmed.ends_with('{');

        // ── Emit __t() BEFORE control-flow headers ────────────────
        // Note: loop headers are handled AFTER (loop var not in scope before)

        let needs_before = is_return || is_if_header || is_else_header;
        if needs_before {
            let visible_vars = get_visible_vars(var_decls, i + 1);
            if !visible_vars.is_empty() {
                let names: Vec<String> = visible_vars
                    .iter()
                    .map(|v| format!("\"{}\"", v.name))
                    .collect();
                let values: Vec<String> = visible_vars.iter().map(|v| v.name.clone()).collect();
                out.push_str(&format!(
                    "        __t({}, new String[]{{{}}}, {});\n",
                    i + 1,
                    names.join(", "),
                    values.join(", ")
                ));
            }
        }

        // ── Emit the original line ───────────────────────────────

        out.push_str(line);
        out.push('\n');

        // ── Emit __t() AFTER statements and loop headers ───

        if is_statement || is_loop_header {
            let visible_vars = get_visible_vars(var_decls, i + 1);
            if !visible_vars.is_empty() {
                let names: Vec<String> = visible_vars
                    .iter()
                    .map(|v| format!("\"{}\"", v.name))
                    .collect();
                let values: Vec<String> = visible_vars.iter().map(|v| v.name.clone()).collect();
                out.push_str(&format!(
                    "        __t({}, new String[]{{{}}}, {});\n",
                    i + 1,
                    names.join(", "),
                    values.join(", ")
                ));
            }
        }
    }

    // Write remaining lines after method
    for i in (body_end + 1)..code_lines.len() {
        out.push_str(&code_lines[i]);
        out.push('\n');
    }

    // Inject __t helper method before the class closes
    // Remove the last '}' (class close), insert helper, then close
    out = out.trim_end().to_string();
    if out.ends_with('}') {
        out.pop();
        out.push('\n');
    }

    // Add the __t helper method
    out.push_str(r#"
    private static void __t(int line, String[] names, Object... values) {
        StringBuilder sb = new StringBuilder();
        sb.append("__TRACE__{\"line\":").append(line).append(",\"vars\":{");
        for (int __i = 0; __i < names.length; __i++) {
            if (__i > 0) sb.append(",");
            sb.append("\"").append(names[__i]).append("\":\"");
            Object __v = values[__i];
            if (__v == null) sb.append("null");
            else if (__v instanceof int[]) sb.append(java.util.Arrays.toString((int[])__v));
            else if (__v instanceof char[]) sb.append(java.util.Arrays.toString((char[])__v));
            else if (__v instanceof boolean[]) sb.append(java.util.Arrays.toString((boolean[])__v));
            else if (__v instanceof String[]) sb.append(java.util.Arrays.toString((String[])__v));
            else if (__v instanceof int[][]) sb.append(java.util.Arrays.deepToString((int[][])__v));
            else {
                String __s = __v.toString();
                __s = __s.replace("\\", "\\\\").replace("\"", "\\\"");
                sb.append(__s);
            }
            sb.append("\"");
        }
        sb.append("}}");
        System.out.println(sb.toString());
    }
}
"#);

    Ok(out)
}

fn get_visible_vars(
    var_decls: &[super::analyzer::VarDecl],
    current_line: usize,
) -> Vec<super::analyzer::VarDecl> {
    var_decls
        .iter()
        .filter(|v| v.line <= current_line && v.scope_end > current_line)
        .cloned()
        .collect()
}

fn generate_runner(
    analysis: &Analysis,
    params: &[(String, TypedValue)],
) -> Result<String, String> {
    let primary = analysis
        .public_methods
        .first()
        .ok_or("没有找到 public 方法")?;

    let mut out = String::new();
    out.push_str("class TraceRunner {\n");

    // Tree builder helper (if needed)
    if analysis.needs_types.contains(&"TreeNode".to_string()) {
        out.push_str(r#"
    static TreeNode buildTree(Integer[] vals) {
        if (vals == null || vals.length == 0 || vals[0] == null) return null;
        TreeNode root = new TreeNode(vals[0]);
        java.util.Queue<TreeNode> q = new java.util.LinkedList<>();
        q.offer(root);
        int i = 1;
        while (!q.isEmpty() && i < vals.length) {
            TreeNode node = q.poll();
            if (vals[i] != null) {
                node.left = new TreeNode(vals[i]);
                q.offer(node.left);
            }
            i++;
            if (i < vals.length && vals[i] != null) {
                node.right = new TreeNode(vals[i]);
                q.offer(node.right);
            }
            i++;
        }
        return root;
    }
"#);
    }

    // ListNode builder
    if analysis.needs_types.contains(&"ListNode".to_string()) {
        out.push_str(r#"
    static ListNode buildList(int[] vals) {
        if (vals == null || vals.length == 0) return null;
        ListNode head = new ListNode(vals[0]);
        ListNode cur = head;
        for (int i = 1; i < vals.length; i++) {
            cur.next = new ListNode(vals[i]);
            cur = cur.next;
        }
        return head;
    }
"#);
    }

    // main method
    out.push_str("\n    public static void main(String[] args) {\n");

    // Build input parameters
    for (param_name, value) in params {
        out.push_str(&format!("        {}\n", value.to_java_init(param_name)));
    }

    // Call the solution
    let method_name = &primary.name;
    let arg_names: Vec<String> = params.iter().map(|(name, _)| name.clone()).collect();
    let return_type = &primary.return_type;

    if return_type == "void" {
        out.push_str(&format!(
            "        new {}().{}({});\n",
            analysis.class_name,
            method_name,
            arg_names.join(", ")
        ));
        out.push_str("        System.out.println(\"__RESULT__void\");\n");
    } else {
        out.push_str(&format!(
            "        {} __result = new {}().{}({});\n",
            boxed_type(return_type),
            analysis.class_name,
            method_name,
            arg_names.join(", ")
        ));
        // Print result with appropriate formatting
        if return_type.contains("[]") {
            out.push_str(
                "        System.out.println(\"__RESULT__\" + java.util.Arrays.toString(__result));\n",
            );
        } else if return_type.contains("List") {
            out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
        } else if return_type == "boolean" {
            out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
        } else if return_type == "int" || return_type == "double" {
            out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
        } else {
            // Object types (ListNode, TreeNode, String)
            out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
        }
    }

    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

fn boxed_type(java_type: &str) -> &str {
    match java_type {
        "int[]" => "int[]",
        "int[][]" => "int[][]",
        "String[]" => "String[]",
        "char[]" => "char[]",
        "boolean" => "boolean",
        _ => java_type,
    }
}
