use clap::{CommandFactory, Parser};
use clap_complete::generate;
use colored::{Color, Colorize};
use leetcode_helper::{Database, Problem};
use std::{io::IsTerminal, process};

mod cli;
mod config;
mod highlight;
mod instrument;
mod theme;
mod trace;
mod tui;

use cli::{Cli, Commands};
pub(crate) use theme::{
    load_theme, MarkdownTheme, SyntaxTheme, TraceTheme, TuiTheme,
};

const EMBEDDED_DATA: &str = include_str!("../data/problems.json");

fn main() {
    let cli = Cli::parse();

    // Handle subcommands first
    if let Some(Commands::Completions { shell }) = &cli.command {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        match shell {
            cli::Shell::Bash => generate(clap_complete::shells::Bash, &mut cmd, &name, &mut std::io::stdout()),
            cli::Shell::Zsh => generate(clap_complete::shells::Zsh, &mut cmd, &name, &mut std::io::stdout()),
            cli::Shell::Fish => generate(clap_complete::shells::Fish, &mut cmd, &name, &mut std::io::stdout()),
            cli::Shell::Powershell => generate(clap_complete::shells::PowerShell, &mut cmd, &name, &mut std::io::stdout()),
        }
        return;
    }

    let color_enabled = std::io::stdout().is_terminal();
    colored::control::set_override(color_enabled);

    // Load global config (CLI args take precedence)
    let config = config::load_config();

    // Use config default theme if not specified via CLI
    let effective_theme = cli.theme.as_deref().or(config.default_theme.as_deref());

    // Apply config cache_dir to LH_CACHE_DIR if not already set
    if let Some(ref dir) = config.cache_dir
        && std::env::var("LH_CACHE_DIR").is_err()
    {
        // safe: we checked LH_CACHE_DIR is not set, so we're not overwriting
        unsafe { std::env::set_var("LH_CACHE_DIR", dir); }
    }

    if cli.list && cli.query.is_some() {
        eprintln!("使用 -l/--list 时不要提供题号或关键词。");
        process::exit(2);
    }

    if cli.search && cli.query.is_none() {
        eprintln!("使用 -s/--search 必须提供关键词。");
        process::exit(2);
    }

    let db = Database::from_json_str(EMBEDDED_DATA);

    let db = match db {
        Ok(db) => db,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    if cli.list {
        print_list(&db);
        return;
    }

    let query = match cli.query {
        Some(q) => q,
        None => {
            eprintln!("请输入题号或关键词，或使用 -l/--list。");
            process::exit(2);
        }
    };

    let treat_as_id = !cli.search && query.chars().all(|c| c.is_ascii_digit());

    if treat_as_id {
        let show_hint = cli.hint;
        let show_answer = cli.answer;
        let show_extra = cli.extra;
        let show_trace = cli.trace;

        if !show_hint && !show_answer && !show_extra && !show_trace {
            eprintln!("请显式指定 -i/--hint、-a/--answer、-e/--extra 或 -t/--trace。");
            process::exit(2);
        }
        match db.get_by_id(&query) {
            Some(problem) => {
                // TUI mode: launch interactive trace viewer
                if show_trace && !cli.trace_text {
                    let theme = load_theme(effective_theme);
                    handle_tui_trace(problem, cli.re_trace, cli.input.as_deref(), &theme.tui);
                    return;
                }
                let output = format_problem(
                    problem,
                    show_hint,
                    show_extra,
                    show_answer,
                    show_trace,
                    cli.re_trace,
                    cli.input.as_deref(),
                    color_enabled,
                    effective_theme,
                );
                println!("{}", output.trim_end());
            }
            None => {
                eprintln!("未找到题号 {}。可以使用 -s/--search 或 -l/--list。", query);
                process::exit(1);
            }
        }
    } else {
        let results = db.search(&query);
        if results.is_empty() {
            eprintln!("关键词 \"{}\" 没有匹配结果。可以使用 -l/--list。", query);
            process::exit(1);
        }

        println!("关键词 \"{}\" 匹配到 {} 条：", query, results.len());
        for problem in results {
            println!("- {}: {}", problem.id, problem.title);
        }
    }
}

fn print_list(db: &Database) {
    for problem in db.list_sorted() {
        println!("{}: {}", problem.id, problem.title);
    }
}

/// Handle the TUI trace flow: generate trace → analyze code → launch TUI.
/// For TUI mode, we prefer auto-instrumentation (denser steps) over static traces.
fn handle_tui_trace(problem: &Problem, re_trace: bool, custom_input: Option<&str>, tui_theme: &TuiTheme) {
    // Always try auto-instrumentation or cache for TUI (more steps than static traces)
    if custom_input.is_none() {
        eprintln!("正在生成执行追踪...");
    } else {
        eprintln!("正在使用自定义输入生成追踪...");
    }
    let trace = match instrument::run_auto_trace(problem, re_trace, custom_input) {
        Ok(t) => t,
        Err(err) => {
            // Fall back to static trace if auto-instrumentation fails
            if let Some(ref static_trace) = problem.trace {
                eprintln!("自动追踪失败，使用预置静态数据: {}", err);
                static_trace.clone()
            } else {
                eprintln!("自动追踪失败: {}", err);
                return;
            }
        }
    };

    // Analyze the answer code for TUI code display
    let analysis = match problem.answer.as_ref() {
        Some(answer) => match instrument::analyzer::analyze(answer) {
            Ok(a) => a,
            Err(err) => {
                eprintln!("分析代码失败: {}", err);
                return;
            }
        },
        None => {
            eprintln!("此题目没有答案代码");
            return;
        }
    };

    // Launch TUI (blocks until user quits)
    if let Err(err) = tui::run_tui(&trace, &analysis, tui_theme) {
        eprintln!("TUI 错误: {}", err);
    }
}

fn format_problem(
    problem: &Problem,
    show_hint: bool,
    show_extra: bool,
    show_answer: bool,
    show_trace: bool,
    re_trace: bool,
    custom_input: Option<&str>,
    color: bool,
    theme_path: Option<&str>,
) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "{} {}\n",
        label("题目:", color),
        format!("{}. {}", problem.id, problem.title)
            .bold()
            .bright_yellow()
            .to_string()
    ));

    if show_hint {
        out.push_str(&format!("{}\n", label("提示:", color)));
        if !problem.category.trim().is_empty() {
            let theme = load_theme(theme_path);
            out.push_str(&format!(
                "{} {}\n",
                label("分类:", color),
                inline_render(&problem.category, color, &theme.markdown)
            ));
        }

        out.push_str(&format!("{} ", label("解法:", color)));
        let sol_rendered = render_value(&problem.solution, color, theme_path);
        out.push_str(&sol_rendered);
        out.push('\n');
        append_section_compact(&mut out, "题目描述", &problem.description, color, theme_path, false);
        append_section_compact(&mut out, "题目本质", &problem.essence, color, theme_path, false);
        append_section_compact(&mut out, "现实类比", &problem.analogy, color, theme_path, false);
        append_section_compact(&mut out, "容器选择", &problem.container, color, theme_path, false);
        append_steps(&mut out, "三步主线:", &problem.steps, color, theme_path);
        append_section_compact(&mut out, "复杂度分析", &problem.complexity, color, theme_path, false);
    }

    if show_extra {
        out.push_str(&format!("{}\n", label("扩展信息:", color)));
        append_section_compact(&mut out, "实际示例", &problem.example, color, theme_path, false);
        append_section_compact(&mut out, "图示说明", &problem.diagram, color, theme_path, false);
        append_api_notes(&mut out, &problem.api_notes_view(), color, theme_path);
    }

    if show_trace {
        let theme = load_theme(theme_path);
        // Use static trace only if not forcing re-trace AND no custom input
        let use_static = !re_trace && custom_input.is_none() && problem.trace.is_some();
        match use_static {
            true => {
                let trace_data = problem.trace.as_ref().unwrap();
                out.push_str(&trace::format_trace(
                    trace_data,
                    &theme.syntax,
                    &theme.trace,
                    color,
                ));
                out.push('\n');
            }
            false => {
                // Try auto-instrumentation (or cache)
                out.push_str(&format!("{} (自动生成中...)\n", label("执行追踪:", color)));
                match instrument::run_auto_trace(problem, re_trace, custom_input) {
                    Ok(trace_data) => {
                        out.push_str(&trace::format_trace(
                            &trace_data,
                            &theme.syntax,
                            &theme.trace,
                            color,
                        ));
                        out.push('\n');
                    }
                    Err(err) => {
                        out.push_str(&format!("自动追踪失败: {}\n", err));
                    }
                }
            }
        }
    }

    if show_answer {
        out.push_str(&format!("{}\n", label("答案代码:", color)));
        let answer = problem.answer.as_deref().unwrap_or("");
        out.push_str(&render_code_block(answer, "java", color, theme_path));
        out.push('\n');
    }

    out
}

fn append_api_notes(out: &mut String, api_notes: &[ApiNoteView], color: bool, theme_path: Option<&str>) {
    if api_notes.is_empty() {
        return;
    }
    let theme = load_theme(theme_path);
    out.push_str(&format!("{}", label("API 注释:", color)));
    out.push('\n');
    for note in api_notes {
        let api_name = if color {
            note.api.color(theme.api.api_name).bold().to_string()
        } else {
            note.api.clone()
        };
        out.push_str(&format!("- {}", api_name));
        if !note.usage.trim().is_empty() {
            let usage_label = if color {
                " 用法: ".color(theme.api.usage_label).to_string()
            } else {
                " 用法: ".to_string()
            };
            out.push_str(&usage_label);
            out.push_str(note.usage.trim());
        }
        if !note.note.trim().is_empty() {
            let note_label = if color {
                " 说明: ".color(theme.api.note_label).to_string()
            } else {
                " 说明: ".to_string()
            };
            out.push_str(&note_label);
            out.push_str(note.note.trim());
        }
        out.push('\n');
    }
}

#[derive(Clone)]
struct ApiNoteView {
    api: String,
    usage: String,
    note: String,
}

trait ProblemApiNotesView {
    fn api_notes_view(&self) -> Vec<ApiNoteView>;
}

impl ProblemApiNotesView for Problem {
    fn api_notes_view(&self) -> Vec<ApiNoteView> {
        self.api_notes
            .iter()
            .map(|item| ApiNoteView {
                api: item.api.clone(),
                usage: item.usage.clone(),
                note: item.note.clone(),
            })
            .collect()
    }
}


fn append_section_compact(
    out: &mut String,
    title: &str,
    value: &str,
    color: bool,
    theme_path: Option<&str>,
    collapse_blank_lines: bool,
) {
    if value.trim().is_empty() {
        return;
    }
    let theme = load_theme(theme_path);
    let label_str = if color {
        format!("{}:", title).color(theme.markdown.section_label).bold().to_string()
    } else {
        format!("{}:", title)
    };
    out.push_str(&label_str);
    if collapse_blank_lines {
        out.push(' ');
    } else {
        out.push('\n');
    }
    let rendered = if collapse_blank_lines {
        render_value_compact(value, color, theme_path)
    } else {
        render_value(value, color, theme_path)
    };
    out.push_str(&rendered);
    out.push('\n');
}

fn append_steps(out: &mut String, title: &str, steps: &[String], color: bool, theme_path: Option<&str>) {
    out.push_str(&format!("{}\n", label(title, color)));
    if steps.is_empty() {
        out.push_str("暂无\n");
        return;
    }

    for item in steps {
        let rendered = render_value(item, color, theme_path);
        out.push_str("- ");
        out.push_str(&rendered);
        out.push('\n');
    }
}

fn label(text: &str, color: bool) -> String {
    if color {
        text.bold().bright_green().to_string()
    } else {
        text.to_string()
    }
}

fn render_value(value: &str, color: bool, theme_path: Option<&str>) -> String {
    if value.trim().is_empty() {
        return "暂无".to_string();
    }

    let md_like = value.contains("```")
        || value.contains("`")
        || value.contains("**")
        || value.contains("# ")
        || value.contains("[")
        || value.contains("> ");

    if !color && !md_like {
        return value.trim_end().to_string();
    }

    let mut out = String::new();
    let theme = load_theme(theme_path);
    let mut in_code = false;
    let mut in_block_comment = false;
    let mut current_lang = String::new();

    for line in value.trim_end().lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_code = !in_code;
            if in_code {
                current_lang = t.trim_start_matches('`').trim().to_ascii_lowercase();
                in_block_comment = false;
            } else {
                current_lang.clear();
            }
            continue;
        }
        if in_code {
            if color {
                out.push_str("    ");
                out.push_str(&highlight::highlight_code_line(line, &theme.syntax, &mut in_block_comment));
                out.push('\n');
            } else {
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }

        if t.is_empty() {
            out.push('\n');
            continue;
        }

        if let Some(rest) = t.strip_prefix("# ") {
            out.push_str(&format!("{}\n", rest.color(theme.markdown.h1).bold()));
            continue;
        }
        if let Some(rest) = t.strip_prefix("## ") {
            out.push_str(&format!("{}\n", rest.color(theme.markdown.h2).bold()));
            continue;
        }
        if let Some(rest) = t.strip_prefix("### ") {
            out.push_str(&format!("{}\n", rest.color(theme.markdown.h3).bold()));
            continue;
        }
        if let Some(rest) = t.strip_prefix("> ") {
            out.push_str(&format!("{}\n", rest.color(theme.markdown.blockquote).dimmed()));
            continue;
        }

        out.push_str(&inline_render(t, color, &theme.markdown));
        out.push('\n');
    }

    out.trim_end().to_string()
}

fn render_value_compact(value: &str, color: bool, theme_path: Option<&str>) -> String {
    let rendered = render_value(value, color, theme_path);
    let mut out = String::new();
    let mut in_code = false;

    for line in rendered.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code = !in_code;
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if !in_code && line.trim().is_empty() {
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    out.trim_end().to_string()
}

fn inline_render(s: &str, color: bool, theme: &MarkdownTheme) -> String {
    if !color {
        return s.to_string();
    }

    let mut out = String::new();
    let mut rest = s;
    while !rest.is_empty() {
        if rest.starts_with("**") {
            if let Some(pos) = rest[2..].find("**") {
                let inner = &rest[2..2 + pos];
                out.push_str(&inner.color(theme.bold).bold().to_string());
                rest = &rest[2 + pos + 2..];
                continue;
            }
        }

        if rest.starts_with('`') {
            if let Some(pos) = rest[1..].find('`') {
                let inner = &rest[1..1 + pos];
                out.push_str(&inner.color(theme.inline_code).to_string());
                rest = &rest[1 + pos + 1..];
                continue;
            }
        }

        if rest.starts_with('[') {
            if let Some(cl_br) = rest.find(']') {
                if rest.get(cl_br + 1..cl_br + 2) == Some("(") {
                    if let Some(cl_par) = rest[cl_br + 2..].find(')') {
                        let text = &rest[1..cl_br];
                        let url = &rest[cl_br + 2..cl_br + 2 + cl_par];
                        out.push_str(&text.underline().color(theme.link).to_string());
                        out.push_str(&format!("({})", url.color(Color::BrightBlack)));
                        rest = &rest[cl_br + 2 + cl_par + 1..];
                        continue;
                    }
                }
            }
        }

        let ch = rest.chars().next().unwrap();
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }

    out
}

fn render_code_block(value: &str, _lang: &str, color: bool, theme_path: Option<&str>) -> String {
    if value.trim().is_empty() {
        return "暂无".to_string();
    }

    let theme = load_theme(theme_path);
    let mut in_block_comment = false;
    let mut out = String::new();
    for (idx, line) in value.trim_end().lines().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        if color {
            out.push_str("    ");
            out.push_str(&highlight::highlight_code_line(
                line,
                &theme.syntax,
                &mut in_block_comment,
            ));
        } else {
            out.push_str(line);
        }
    }
    out
}
