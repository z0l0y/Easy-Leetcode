pub mod analyzer;
pub mod compiler;
pub mod example;
pub mod generator;
pub mod trace_parser;

use leetcode_helper::{Problem, Trace};
use std::path::PathBuf;

/// Run the full auto-instrumentation pipeline for a problem.
/// If `force` is true, skips the cache and regenerates.
/// If `custom_input` is provided, uses it instead of `problem.example`.
/// Returns a Trace if successful, or an error message string.
pub fn run_auto_trace(
    problem: &Problem,
    force: bool,
    custom_input: Option<&str>,
) -> Result<Trace, String> {
    // Custom input bypasses cache (different inputs → different traces)
    if !force && custom_input.is_none() {
        if let Some(cached) = load_cached_trace(&problem.id) {
            return Ok(cached);
        }
    }

    let answer = problem.answer.as_ref().ok_or("此题目没有答案代码")?;

    // Use custom input if provided, otherwise fall back to example
    let example_text = custom_input
        .map(|s| format!("输入：{}", s))
        .unwrap_or_else(|| problem.example.clone());

    if example_text.trim().is_empty() {
        return Err("此题目没有示例数据（可用 --input 指定）".into());
    }

    // Phase 1: Parse example input
    let params = example::parse_example(&example_text)?;

    // Phase 2: Analyze Java code
    let analysis = analyzer::analyze(answer)?;

    // Check for unsupported class patterns
    if analysis.class_name != "Solution" && analysis.public_methods.len() > 1 {
        return Err(format!(
            "此题目({})暂不支持自动追踪（多方法类）",
            analysis.class_name
        ));
    }

    // Phase 3: Generate instrumented Java
    let java_code = generator::generate(&analysis, &params)?;

    // Phase 4: Compile and run
    let result = compiler::compile_and_run(&java_code, "TraceRunner")?;

    // Phase 5: Parse output into Trace
    let trace = trace_parser::parse(&result.output, answer, &analysis, &params)?;

    // Save to cache (only for default example, not custom input)
    if custom_input.is_none() {
        let _ = save_trace_cache(&problem.id, &trace);
    }

    Ok(trace)
}

// ─── Trace cache ──────────────────────────────────────────────────────

fn cache_path(problem_id: &str) -> PathBuf {
    PathBuf::from("data/traces").join(format!("{}.json", problem_id))
}

fn load_cached_trace(problem_id: &str) -> Option<Trace> {
    let path = cache_path(problem_id);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str::<Trace>(&content).ok()
}

fn save_trace_cache(problem_id: &str, trace: &Trace) -> Result<(), String> {
    let path = cache_path(problem_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建缓存目录失败: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(trace).map_err(|e| format!("序列化 trace 失败: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("写入缓存失败: {}", e))?;
    Ok(())
}
