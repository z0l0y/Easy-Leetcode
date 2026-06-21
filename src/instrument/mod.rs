pub mod analyzer;
pub mod compiler;
pub mod example;
pub mod generator;
pub mod trace_parser;

use anyhow::Context;
use leetcode_helper::{Problem, Trace};
use std::path::PathBuf;

/// Run the full auto-instrumentation pipeline for a problem.
/// If `force` is true, skips the cache and regenerates.
/// If `custom_input` is provided, uses it instead of `problem.example`.
/// Returns a Trace if successful.
pub fn run_auto_trace(
    problem: &Problem,
    force: bool,
    custom_input: Option<&str>,
) -> anyhow::Result<Trace> {
    // Custom input bypasses cache (different inputs → different traces)
    if !force && custom_input.is_none() {
        if let Some(cached) = load_cached_trace(&problem.id) {
            return Ok(cached);
        }
    }

    let answer = problem.answer.as_ref().context("此题目没有答案代码")?;

    // Use custom input if provided, otherwise fall back to example
    let example_text = custom_input
        .map(|s| format!("输入：{}", s))
        .unwrap_or_else(|| problem.example.clone());

    if example_text.trim().is_empty() {
        anyhow::bail!("此题目没有示例数据（可用 --input 指定）");
    }

    // Phase 1: Parse example input
    let example_input = example::parse_example_input(&example_text)
        .context("解析示例输入失败")?;

    // Phase 2: Analyze Java code
    let analysis = analyzer::analyze(answer)
        .context("分析 Java 代码失败")?;

    // Multi-method classes (e.g. LRUCache, MinStack) are now supported.
    // The runner will generate operation-sequence based main().

    // Phase 3: Generate instrumented Java
    let java_code = generator::generate(&analysis, &example_input)
        .context("生成插桩代码失败")?;

    // Phase 4: Compile and run
    let result = compiler::compile_and_run(&java_code, "TraceRunner")
        .context("编译或运行 Java 代码失败")?;

    // Phase 5: Parse output into Trace
    let trace = trace_parser::parse(&result.output, answer, &analysis, &example_input)
        .context("解析执行追踪数据失败")?;

    // Save to cache (only for default example, not custom input)
    if custom_input.is_none() {
        let _ = save_trace_cache(&problem.id, &trace);
    }

    Ok(trace)
}

// ─── Trace cache ──────────────────────────────────────────────────────

fn cache_path(problem_id: &str) -> PathBuf {
    // Check LH_CACHE_DIR environment variable first
    if let Ok(dir) = std::env::var("LH_CACHE_DIR") {
        return PathBuf::from(dir).join(format!("{}.json", problem_id));
    }
    PathBuf::from("data/traces").join(format!("{}.json", problem_id))
}

fn load_cached_trace(problem_id: &str) -> Option<Trace> {
    let path = cache_path(problem_id);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str::<Trace>(&content).ok()
}

fn save_trace_cache(problem_id: &str, trace: &Trace) -> anyhow::Result<()> {
    let path = cache_path(problem_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("创建缓存目录失败")?;
    }
    let json = serde_json::to_string_pretty(trace).context("序列化 trace 失败")?;
    std::fs::write(&path, json).context("写入缓存失败")?;
    Ok(())
}
