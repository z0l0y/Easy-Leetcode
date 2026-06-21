use super::analyzer::Analysis;
use super::example::ExampleInput;
use anyhow::Context;

/// Generate a complete, compilable TraceRunner.java file.
pub fn generate(analysis: &Analysis, input: &ExampleInput) -> anyhow::Result<String> {
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
        anyhow::bail!("没有找到 public 方法")
    }

    // 3. The TraceRunner class with main()
    out.push('\n');
    out.push_str(&generate_runner(analysis, input)?);

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
        int limit = 100;
        while (cur != null && limit > 0) {
            sb.append(cur.val);
            if (cur.next != null) sb.append("->");
            cur = cur.next;
            limit--;
        }
        if (cur != null) sb.append("...");
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
    public String toLevelOrder() {
        StringBuilder sb = new StringBuilder();
        sb.append("[");
        java.util.Queue<TreeNode> q = new java.util.LinkedList<>();
        q.offer(this);
        int nonNullCount = 0;
        while (!q.isEmpty()) {
            TreeNode node = q.poll();
            if (node == null) {
                if (nonNullCount > 0) sb.append("null,");
                continue;
            }
            sb.append(node.val).append(",");
            nonNullCount--;
            q.offer(node.left);
            q.offer(node.right);
            if (node.left != null) nonNullCount = q.size();
            if (node.right != null) nonNullCount = q.size();
        }
        if (sb.length() > 1) sb.setLength(sb.length() - 1); // remove trailing comma
        sb.append("]");
        return sb.toString();
    }
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

/// Build the __t / __t_enter / __t_exit helper methods.
/// Only includes type-specific instanceof checks for types that are actually defined.
fn generate_t_helper(analysis: &Analysis) -> String {
    let has_tree = analysis.needs_types.contains(&"TreeNode".to_string());

    let mut out = String::new();
    out.push_str("\n");
    out.push_str("    private static String __current_method = \"\";\n");
    out.push_str("    private static java.util.Stack<String> __call_stack = new java.util.Stack<>();\n");
    out.push_str("\n");
    out.push_str("    private static void __t_enter(String method) {\n");
    out.push_str("        __current_method = method;\n");
    out.push_str("        __call_stack.push(method);\n");
    out.push_str("    }\n");
    out.push_str("\n");
    out.push_str("    private static void __t_exit() {\n");
    out.push_str("        if (!__call_stack.isEmpty()) __call_stack.pop();\n");
    out.push_str("        __current_method = __call_stack.isEmpty() ? \"\" : __call_stack.peek();\n");
    out.push_str("    }\n");
    out.push_str("\n");
    out.push_str("    private static void __t(int line, String[] names, Object[] values) {\n");
    out.push_str("        StringBuilder sb = new StringBuilder();\n");
    out.push_str("        sb.append(\"__TRACE__{\\\"line\\\":\").append(line);\n");
    out.push_str("        if (!__current_method.isEmpty()) {\n");
    out.push_str("            sb.append(\",\\\"method\\\":\\\"\").append(__current_method).append(\"\\\"\");\n");
    out.push_str("        }\n");
    out.push_str("        if (__call_stack.size() > 1) {\n");
    out.push_str("            sb.append(\",\\\"stack\\\":[\");\n");
    out.push_str("            for (int __si = 0; __si < __call_stack.size(); __si++) {\n");
    out.push_str("                if (__si > 0) sb.append(\",\");\n");
    out.push_str("                sb.append(\"\\\"\").append(__call_stack.get(__si)).append(\"\\\"\");\n");
    out.push_str("            }\n");
    out.push_str("            sb.append(\"]\");\n");
    out.push_str("        }\n");
    out.push_str("        sb.append(\",\\\"vars\\\":{\");\n");
    out.push_str("        for (int __i = 0; __i < names.length; __i++) {\n");
    out.push_str("            if (__i > 0) sb.append(\",\");\n");
    out.push_str("            sb.append(\"\\\"\").append(names[__i]).append(\"\\\":\\\"\");\n");
    out.push_str("            Object __v = values[__i];\n");
    out.push_str("            if (__v == null) sb.append(\"null\");\n");
    out.push_str("            else if (__v instanceof int[]) sb.append(java.util.Arrays.toString((int[])__v));\n");
    out.push_str("            else if (__v instanceof long[]) sb.append(java.util.Arrays.toString((long[])__v));\n");
    out.push_str("            else if (__v instanceof double[]) sb.append(java.util.Arrays.toString((double[])__v));\n");
    out.push_str("            else if (__v instanceof char[]) sb.append(java.util.Arrays.toString((char[])__v));\n");
    out.push_str("            else if (__v instanceof boolean[]) sb.append(java.util.Arrays.toString((boolean[])__v));\n");
    out.push_str("            else if (__v instanceof String[]) sb.append(java.util.Arrays.toString((String[])__v));\n");
    out.push_str("            else if (__v instanceof int[][]) sb.append(java.util.Arrays.deepToString((int[][])__v));\n");
    out.push_str("            else if (__v instanceof java.util.Collection) sb.append(__v.toString());\n");
    out.push_str("            else if (__v instanceof java.util.Map) sb.append(__v.toString());\n");
    if has_tree {
        out.push_str("            else if (__v instanceof TreeNode) sb.append(((TreeNode)__v).toLevelOrder());\n");
    }
    out.push_str("            else {\n");
    out.push_str("                String __s = __v.toString();\n");
    out.push_str("                __s = __s.replace(\"\\\\\", \"\\\\\\\\\").replace(\"\\\"\", \"\\\\\\\"\");\n");
    out.push_str("                sb.append(__s);\n");
    out.push_str("            }\n");
    out.push_str("            sb.append(\"\\\"\");\n");
    out.push_str("        }\n");
    out.push_str("        sb.append(\"}}\");\n");
    out.push_str("        System.out.println(sb.toString());\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out
}

fn generate_instrumented_solution(
    analysis: &Analysis,
    method: &super::analyzer::MethodInfo,
    var_decls: &[super::analyzer::VarDecl],
) -> anyhow::Result<String> {
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

        // If this is the opening brace of the method, insert method entry + initial capture
        if i == body_start {
            // Method entry tracking
            out.push_str(&format!(
                "        __t_enter(\"{}\");\n",
                method.name
            ));

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
                    "        __t({}, new String[]{{{}}}, new Object[]{{{}}});\n",
                    i + 1,
                    param_names.join(", "),
                    param_values.join(", ")
                ));
            }

            // Wrap method body in try-finally so __t_exit() runs on every exit path
            out.push_str("        try {\n");
        }
    }

    // Write instrumented body lines — up to (but NOT including) the method's closing `}`
    // __t_exit() must go BEFORE the closing brace so it's inside the method body.
    for i in (body_start + 1)..body_end.min(code_lines.len()) {
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

        // return/break/continue statements: capture BEFORE we leave
        let is_return = trimmed.starts_with("return ") || trimmed == "return;";
        let is_break = trimmed == "break;" || trimmed.starts_with("break ");
        let is_continue = trimmed == "continue;" || trimmed.starts_with("continue ");

        // Any statement ending with ; (assignments, method calls, declarations)
        // Exclude control-flow statements (break/continue/return) — they'd make
        // following __t() calls unreachable.
        let is_statement = trimmed.ends_with(';')
            && !is_return && !is_break && !is_continue
            && !trimmed.starts_with("if ")
            && !trimmed.starts_with("for ")
            && !trimmed.starts_with("while ")
            && !trimmed.starts_with("} while");

        // if-condition headers: if (...) {  or  if (...) \n  (single-line if without braces)
        let is_if_header = (trimmed.starts_with("if (") || trimmed.starts_with("if("))
            && (trimmed.ends_with('{') || trimmed.ends_with(')'));

        // else / else-if transitions
        let is_else_header = (trimmed.starts_with("} else") || trimmed.starts_with("}else"))
            && (trimmed.ends_with('{') || trimmed.contains("if ("));

        // Loop headers: for (...) {  or  while (...) {  or  do {
        let is_loop_header = (trimmed.starts_with("for (") || trimmed.starts_with("for(")
            || trimmed.starts_with("while (") || trimmed.starts_with("while("))
            && trimmed.ends_with('{')
            || trimmed == "do {" || trimmed.starts_with("do {");

        // do-while end: } while (...);
        let is_do_while_end = trimmed.starts_with("} while (") || trimmed.starts_with("}while(");

        // switch header: switch (...) {
        let is_switch_header = (trimmed.starts_with("switch (") || trimmed.starts_with("switch("))
            && trimmed.ends_with('{');

        // case / default labels
        let is_case_label = trimmed.starts_with("case ") || trimmed.starts_with("default:")
            || trimmed.starts_with("default :");

        // try/catch/finally headers
        let is_try_header = trimmed == "try {" || trimmed.starts_with("try {");
        let is_catch_header = (trimmed.starts_with("catch (") || trimmed.starts_with("catch("))
            && (trimmed.ends_with('{') || trimmed.contains(')'));
        let is_finally_header = trimmed == "finally {" || trimmed.starts_with("finally {");

        // ── Emit __t() BEFORE control-flow headers ────────────────
        // Note: loop headers are handled AFTER (loop var not in scope before)
        // Note: case labels are handled BEFORE (capture state entering new case)

        let needs_before = is_return || is_if_header || is_else_header
            || is_switch_header || is_case_label
            || is_try_header || is_catch_header || is_finally_header;
        if needs_before {
            let visible_vars = get_visible_vars(var_decls, i + 1);
            if !visible_vars.is_empty() || is_return {
                let mut names: Vec<String> = visible_vars
                    .iter()
                    .map(|v| format!("\"{}\"", v.name))
                    .collect();
                let mut values: Vec<String> =
                    visible_vars.iter().map(|v| v.name.clone()).collect();

                // For return statements, capture the return expression value
                if is_return {
                    let ret_expr = extract_return_expr(trimmed);
                    names.insert(0, "\"__return__\"".to_string());
                    values.insert(0, ret_expr);
                }

                out.push_str(&format!(
                    "        __t({}, new String[]{{{}}}, new Object[]{{{}}});\n",
                    i + 1,
                    names.join(", "),
                    values.join(", ")
                ));
            }
        }

        // ── Emit the original line ───────────────────────────────

        out.push_str(line);
        out.push('\n');

        // ── Emit __t() AFTER statements, loop headers, and do-while ends ───

        if is_statement || is_loop_header || is_do_while_end {
            let visible_vars = get_visible_vars(var_decls, i + 1);
            if !visible_vars.is_empty() {
                let names: Vec<String> = visible_vars
                    .iter()
                    .map(|v| format!("\"{}\"", v.name))
                    .collect();
                let values: Vec<String> = visible_vars.iter().map(|v| v.name.clone()).collect();
                out.push_str(&format!(
                    "        __t({}, new String[]{{{}}}, new Object[]{{{}}});\n",
                    i + 1,
                    names.join(", "),
                    values.join(", ")
                ));
            }
        }
    }

    // Method exit tracking — try-finally ensures __t_exit() runs on every exit path
    out.push_str("        } finally {\n");
    out.push_str("            __t_exit();\n");
    out.push_str("        }\n");

    // Write the method's closing brace
    if let Some(closing_line) = code_lines.get(body_end) {
        out.push_str(closing_line);
        out.push('\n');
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

    // Add the __t helper method — built dynamically based on needed types
    out.push_str(&generate_t_helper(analysis));

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

/// Extract the expression from a return statement, e.g. "return x + y;" → "x + y".
fn extract_return_expr(trimmed: &str) -> String {
    let s = trimmed.strip_prefix("return ").unwrap_or(trimmed);
    let s = s.strip_suffix(';').unwrap_or(s);
    s.trim().to_string()
}

fn generate_runner(
    analysis: &Analysis,
    input: &ExampleInput,
) -> anyhow::Result<String> {
    let primary = analysis
        .public_methods
        .first()
        .context("没有找到 public 方法")?;

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

    // main method — branches on input type
    out.push_str("\n    public static void main(String[] args) {\n");

    match input {
        ExampleInput::Single(params) => {
            // ── Single-method call ──────────────────────────
            // Look up expected Java types from the method signature
            let method_name = &primary.name;
            let return_type = &primary.return_type;

            // Build input parameters with type-aware initialization.
            // LeetCode examples often have extra metadata params (pos, skipA, etc.)
            // whose names don't match method parameters. Also, example param names
            // may differ from method param names (e.g. listA vs headA).
            //
            // Strategy:
            // 1. First try to match each method param by name in the example params.
            // 2. For unmatched method params, fall back to positional matching:
            //    scan remaining example params for a value whose type is compatible.
            let mut arg_names: Vec<String> = Vec::new();
            let mut used_example_indices: Vec<bool> = vec![false; params.len()];

            for (method_type, method_param_name) in &primary.params {
                // 1. Try name match first
                let found = params.iter().enumerate().find(|(idx, (pname, _))| {
                    !used_example_indices[*idx] && pname == method_param_name
                });
                if let Some((idx, (pname, value))) = found {
                    used_example_indices[idx] = true;
                    out.push_str(&format!("        {}\n",
                        value.to_java_init_typed(pname, method_type.as_str())));
                    arg_names.push(pname.clone());
                    continue;
                }

                // 2. Fallback: find compatible value by type
                let type_fallback = params.iter().enumerate().find(|(idx, (_, value))| {
                    if used_example_indices[*idx] { return false; }
                    is_value_compatible_with_type(value, method_type.as_str())
                });
                if let Some((idx, (_, value))) = type_fallback {
                    used_example_indices[idx] = true;
                    out.push_str(&format!("        {}\n",
                        value.to_java_init_typed(method_param_name, method_type.as_str())));
                    arg_names.push(method_param_name.clone());
                }
                // If still no match: method param is skipped (call will fail at javac)
            }

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
                if return_type.contains("[]") {
                    out.push_str(
                        "        System.out.println(\"__RESULT__\" + java.util.Arrays.toString(__result));\n",
                    );
                } else if return_type.contains("List") || return_type.contains("List") {
                    out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
                } else if return_type == "boolean" {
                    out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
                } else if return_type == "int" || return_type == "double" {
                    out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
                } else {
                    out.push_str("        System.out.println(\"__RESULT__\" + __result);\n");
                }
            }
        }
        ExampleInput::Operations(_class_name, operations) => {
            // ── Multi-method operation sequence ──────────────
            // Create the instance once
            let first_op = &operations[0];
            let ctor_args: Vec<String> = first_op.1.iter()
                .map(|v| v.to_java_literal())
                .collect();
            out.push_str(&format!(
                "        {} obj = new {}({});\n",
                analysis.class_name,
                analysis.class_name,
                ctor_args.join(", ")
            ));

            // Execute each subsequent operation
            for (i, (method_name, args)) in operations.iter().enumerate() {
                if i == 0 {
                    // Constructor call already done
                    continue;
                }
                let arg_strs: Vec<String> = args.iter()
                    .map(|v| v.to_java_literal())
                    .collect();
                // Find the method's return type
                let method = analysis.public_methods.iter()
                    .find(|m| m.name == *method_name);

                let return_type = method.map(|m| m.return_type.as_str()).unwrap_or("void");
                if return_type == "void" {
                    out.push_str(&format!(
                        "        obj.{}({});\n",
                        method_name,
                        arg_strs.join(", ")
                    ));
                } else {
                    out.push_str(&format!(
                        "        {} __r{} = obj.{}({});\n",
                        boxed_type(return_type),
                        i,
                        method_name,
                        arg_strs.join(", ")
                    ));
                    if return_type.contains("[]") {
                        out.push_str(&format!(
                            "        System.out.println(\"__RESULT__\" + java.util.Arrays.toString(__r{}));\n",
                            i
                        ));
                    } else {
                        out.push_str(&format!(
                            "        System.out.println(\"__RESULT__\" + __r{});\n",
                            i
                        ));
                    }
                }
            }
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

/// Check whether a TypedValue is compatible with an expected Java parameter type.
/// Used for fallback matching when example param names differ from method param names.
fn is_value_compatible_with_type(value: &super::example::TypedValue, java_type: &str) -> bool {
    match value {
        super::example::TypedValue::Int(_) => {
            java_type == "int" || java_type == "Integer"
                || java_type == "long" || java_type == "double"
                || java_type == "boolean"
        }
        super::example::TypedValue::String(_) => {
            java_type == "String" || java_type == "char"
        }
        super::example::TypedValue::Bool(_) => {
            java_type == "boolean" || java_type == "Boolean"
        }
        super::example::TypedValue::Array(_) => {
            java_type == "int[]" || java_type == "String[]"
                || java_type == "ListNode" || java_type == "TreeNode"
                || java_type == "ListNode[]"
        }
        super::example::TypedValue::NestedArray(_) => {
            java_type == "int[][]" || java_type == "ListNode[]"
        }
        super::example::TypedValue::TreeNodeArray(_) => {
            java_type == "TreeNode" || java_type == "Integer[]"
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
    fn test_generate_two_sum() {
        let analysis = analyzer::analyze(TWO_SUM).unwrap();
        let input = ExampleInput::Single(vec![
            ("nums".to_string(), TypedValue::Array(vec![TypedValue::Int(2), TypedValue::Int(7)])),
            ("target".to_string(), TypedValue::Int(9)),
        ]);
        let code = generate(&analysis, &input).unwrap();

        // Should contain __t() instrumentation calls
        assert!(code.contains("__t("), "missing __t() calls");
        // Should contain __TRACE__ output
        assert!(code.contains("__TRACE__"), "missing __TRACE__");
        // Should contain __return__ variable capture
        assert!(code.contains("\"__return__\""), "missing __return__ capture");
        // Should contain the __t helper method
        assert!(code.contains("private static void __t("), "missing __t helper");
        // Should contain TraceRunner with main
        assert!(code.contains("class TraceRunner"), "missing TraceRunner");
        assert!(code.contains("public static void main"), "missing main method");
        // Should build and call Solution
        assert!(code.contains("new Solution().twoSum"), "missing Solution invocation");
    }

    #[test]
    fn test_generate_with_return_capture() {
        let analysis = analyzer::analyze(TWO_SUM).unwrap();
        let input = ExampleInput::Single(vec![
            ("nums".to_string(), TypedValue::Array(vec![TypedValue::Int(1)])),
            ("target".to_string(), TypedValue::Int(1)),
        ]);
        let code = generate(&analysis, &input).unwrap();
        // Should capture the return expression before each return statement
        assert!(code.contains("map.get(complement), i"), "missing return expression");
    }
}
