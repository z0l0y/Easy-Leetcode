pub mod analyzer;
pub mod compiler;
pub mod example;
pub mod generator;
pub mod trace_parser;

use leetcode_helper::{Problem, Trace};
use std::path::PathBuf;

/// Run the full auto-instrumentation pipeline for a problem.
/// If `force` is true, skips the cache and regenerates.
/// Returns a Trace if successful, or an error message string.
pub fn run_auto_trace(problem: &Problem, force: bool) -> Result<Trace, String> {
    // Check cache (unless --re-trace)
    if !force {
        if let Some(cached) = load_cached_trace(&problem.id) {
            return Ok(cached);
        }
    }

    let answer = problem.answer.as_ref().ok_or("此题目没有答案代码")?;
    if problem.example.trim().is_empty() {
        return Err("此题目没有示例数据".into());
    }

    // Phase 1: Parse example input
    let params = example::parse_example(&problem.example)?;

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

    // Save to cache
    let _ = save_trace_cache(&problem.id, &trace);

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
