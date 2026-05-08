use clap::Parser;
use colored::{Color, Colorize};
use leetcode_helper::{Database, Problem};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    process,
};

const EMBEDDED_DATA: &str = include_str!("../data/problems.json");

#[derive(Parser, Debug)]
#[command(
    name = "lh",
    version,
    about = "LeetCode Hot100 题解速查工具",
    arg_required_else_help = true
)]
struct Cli {
    #[arg(help = "题号或关键词")]
    query: Option<String>,

    #[arg(short = 'i', long, help = "显示提示内容")]
    hint: bool,

    #[arg(short = 'a', long, help = "显示答案代码")]
    answer: bool,

    #[arg(short = 'e', long, help = "显示扩展信息（示例、图示、API 说明）")]
    extra: bool,

    #[arg(short = 'l', long, help = "列出全部题目")]
    list: bool,

    #[arg(short = 's', long, help = "将输入视为关键词搜索")]
    search: bool,

    #[arg(
        long,
        value_name = "FILE",
        help = "语法高亮主题文件路径（TOML），默认尝试加载 ./theme.toml"
    )]
    theme: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let color_enabled = true;

    colored::control::set_override(true);

    #[cfg(windows)]
    {
        enable_vt_mode();
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

        if !show_hint && !show_answer && !show_extra {
            eprintln!("请显式指定 -i/--hint、-a/--answer 或 -e/--extra。");
            process::exit(2);
        }
        match db.get_by_id(&query) {
            Some(problem) => {
                let output = format_problem(
                    problem,
                    show_hint,
                    show_extra,
                    show_answer,
                    color_enabled,
                    cli.theme.as_deref(),
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

fn format_problem(
    problem: &Problem,
    show_hint: bool,
    show_extra: bool,
    show_answer: bool,
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
                out.push_str(&highlight_code_line(line, &current_lang, &theme.syntax, &mut in_block_comment));
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

fn render_code_block(value: &str, lang: &str, color: bool, theme_path: Option<&str>) -> String {
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
            out.push_str(&highlight_code_line(
                line,
                &lang.to_ascii_lowercase(),
                &theme.syntax,
                &mut in_block_comment,
            ));
        } else {
            out.push_str(line);
        }
    }
    out
}

#[derive(Clone, Deserialize)]
struct ThemeToml {
    syntax: Option<SyntaxThemeToml>,
    markdown: Option<MarkdownThemeToml>,
    api: Option<ApiThemeToml>,
}

#[derive(Clone, Deserialize)]
struct SyntaxThemeToml {
    default: Option<String>,
    keyword: Option<String>,
    type_name: Option<String>,
    function: Option<String>,
    string: Option<String>,
    number: Option<String>,
    comment: Option<String>,
    operator: Option<String>,
    punctuation: Option<String>,
}

#[derive(Clone, Deserialize)]
struct MarkdownThemeToml {
    title: Option<String>,
    section_label: Option<String>,
    code_block: Option<String>,
    inline_code: Option<String>,
    bold: Option<String>,
    link: Option<String>,
    blockquote: Option<String>,
    h1: Option<String>,
    h2: Option<String>,
    h3: Option<String>,
    list_marker: Option<String>,
}

#[derive(Clone, Deserialize)]
struct ApiThemeToml {
    api_name: Option<String>,
    usage_label: Option<String>,
    note_label: Option<String>,
}

#[derive(Clone)]
struct SyntaxTheme {
    default: Color,
    keyword: Color,
    type_name: Color,
    function: Color,
    string: Color,
    number: Color,
    comment: Color,
    operator: Color,
    punctuation: Color,
}

#[derive(Clone)]
struct MarkdownTheme {
    title: Color,
    section_label: Color,
    code_block: Color,
    inline_code: Color,
    bold: Color,
    link: Color,
    blockquote: Color,
    h1: Color,
    h2: Color,
    h3: Color,
    list_marker: Color,
}

#[derive(Clone)]
struct ApiTheme {
    api_name: Color,
    usage_label: Color,
    note_label: Color,
}

#[derive(Clone)]
struct Theme {
    syntax: SyntaxTheme,
    markdown: MarkdownTheme,
    api: ApiTheme,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            default: Color::BrightWhite,
            keyword: Color::BrightYellow,
            type_name: Color::BrightBlue,
            function: Color::BrightCyan,
            string: Color::BrightMagenta,
            number: Color::BrightRed,
            comment: Color::Green,
            operator: Color::BrightWhite,
            punctuation: Color::BrightBlack,
        }
    }
}

impl Default for MarkdownTheme {
    fn default() -> Self {
        Self {
            title: Color::BrightYellow,
            section_label: Color::BrightGreen,
            code_block: Color::BrightCyan,
            inline_code: Color::Cyan,
            bold: Color::BrightWhite,
            link: Color::BrightBlue,
            blockquote: Color::BrightBlack,
            h1: Color::BrightYellow,
            h2: Color::BrightYellow,
            h3: Color::BrightWhite,
            list_marker: Color::Green,
        }
    }
}

impl Default for ApiTheme {
    fn default() -> Self {
        Self {
            api_name: Color::BrightMagenta,
            usage_label: Color::Cyan,
            note_label: Color::Yellow,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            syntax: SyntaxTheme::default(),
            markdown: MarkdownTheme::default(),
            api: ApiTheme::default(),
        }
    }
}

fn load_theme(theme_path: Option<&str>) -> Theme {
    let path = match theme_path {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("theme.toml"),
    };

    let Ok(content) = fs::read_to_string(path) else {
        return Theme::default();
    };

    let Ok(config) = toml::from_str::<ThemeToml>(&content) else {
        return Theme::default();
    };

    let mut theme = Theme::default();

    if let Some(syntax) = config.syntax {
        if let Some(value) = syntax.default.and_then(|s| parse_color_name(&s)) {
            theme.syntax.default = value;
        }
        if let Some(value) = syntax.keyword.and_then(|s| parse_color_name(&s)) {
            theme.syntax.keyword = value;
        }
        if let Some(value) = syntax.type_name.and_then(|s| parse_color_name(&s)) {
            theme.syntax.type_name = value;
        }
        if let Some(value) = syntax.function.and_then(|s| parse_color_name(&s)) {
            theme.syntax.function = value;
        }
        if let Some(value) = syntax.string.and_then(|s| parse_color_name(&s)) {
            theme.syntax.string = value;
        }
        if let Some(value) = syntax.number.and_then(|s| parse_color_name(&s)) {
            theme.syntax.number = value;
        }
        if let Some(value) = syntax.comment.and_then(|s| parse_color_name(&s)) {
            theme.syntax.comment = value;
        }
        if let Some(value) = syntax.operator.and_then(|s| parse_color_name(&s)) {
            theme.syntax.operator = value;
        }
        if let Some(value) = syntax.punctuation.and_then(|s| parse_color_name(&s)) {
            theme.syntax.punctuation = value;
        }
    }

    if let Some(markdown) = config.markdown {
        if let Some(value) = markdown.title.and_then(|s| parse_color_name(&s)) {
            theme.markdown.title = value;
        }
        if let Some(value) = markdown.section_label.and_then(|s| parse_color_name(&s)) {
            theme.markdown.section_label = value;
        }
        if let Some(value) = markdown.code_block.and_then(|s| parse_color_name(&s)) {
            theme.markdown.code_block = value;
        }
        if let Some(value) = markdown.inline_code.and_then(|s| parse_color_name(&s)) {
            theme.markdown.inline_code = value;
        }
        if let Some(value) = markdown.bold.and_then(|s| parse_color_name(&s)) {
            theme.markdown.bold = value;
        }
        if let Some(value) = markdown.link.and_then(|s| parse_color_name(&s)) {
            theme.markdown.link = value;
        }
        if let Some(value) = markdown.blockquote.and_then(|s| parse_color_name(&s)) {
            theme.markdown.blockquote = value;
        }
        if let Some(value) = markdown.h1.and_then(|s| parse_color_name(&s)) {
            theme.markdown.h1 = value;
        }
        if let Some(value) = markdown.h2.and_then(|s| parse_color_name(&s)) {
            theme.markdown.h2 = value;
        }
        if let Some(value) = markdown.h3.and_then(|s| parse_color_name(&s)) {
            theme.markdown.h3 = value;
        }
        if let Some(value) = markdown.list_marker.and_then(|s| parse_color_name(&s)) {
            theme.markdown.list_marker = value;
        }
    }

    if let Some(api) = config.api {
        if let Some(value) = api.api_name.and_then(|s| parse_color_name(&s)) {
            theme.api.api_name = value;
        }
        if let Some(value) = api.usage_label.and_then(|s| parse_color_name(&s)) {
            theme.api.usage_label = value;
        }
        if let Some(value) = api.note_label.and_then(|s| parse_color_name(&s)) {
            theme.api.note_label = value;
        }
    }

    theme
}

fn parse_color_name(name: &str) -> Option<Color> {
    match name.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "white" => Some(Color::White),
        "bright_black" | "brightblack" | "gray" | "grey" => Some(Color::BrightBlack),
        "bright_red" | "brightred" => Some(Color::BrightRed),
        "bright_green" | "brightgreen" => Some(Color::BrightGreen),
        "bright_yellow" | "brightyellow" => Some(Color::BrightYellow),
        "bright_blue" | "brightblue" => Some(Color::BrightBlue),
        "bright_magenta" | "brightmagenta" => Some(Color::BrightMagenta),
        "bright_cyan" | "brightcyan" => Some(Color::BrightCyan),
        "bright_white" | "brightwhite" => Some(Color::BrightWhite),
        _ => None,
    }
}

#[derive(Copy, Clone)]
enum TokenKind {
    Default,
    Keyword,
    TypeName,
    Function,
    String,
    Number,
    Comment,
    Operator,
    Punctuation,
}

fn highlight_code_line(
    line: &str,
    _lang: &str,
    theme: &SyntaxTheme,
    in_block_comment: &mut bool,
) -> String {
    let tokens = lex_code_line(line, in_block_comment);
    let mut out = String::new();
    for (kind, text) in tokens {
        let color = match kind {
            TokenKind::Default => theme.default,
            TokenKind::Keyword => theme.keyword,
            TokenKind::TypeName => theme.type_name,
            TokenKind::Function => theme.function,
            TokenKind::String => theme.string,
            TokenKind::Number => theme.number,
            TokenKind::Comment => theme.comment,
            TokenKind::Operator => theme.operator,
            TokenKind::Punctuation => theme.punctuation,
        };
        out.push_str(&text.color(color).to_string());
    }
    out
}

fn lex_code_line(line: &str, in_block_comment: &mut bool) -> Vec<(TokenKind, String)> {
    let keywords: HashSet<&'static str> = [
        "if", "else", "for", "while", "do", "switch", "case", "break", "continue", "return",
        "try", "catch", "finally", "throw", "throws", "new", "class", "interface", "enum",
        "public", "private", "protected", "static", "final", "abstract", "extends",
        "implements", "import", "package", "void", "this", "super", "true", "false", "null",
    ]
    .into_iter()
    .collect();
    let type_words: HashSet<&'static str> = [
        "int", "long", "double", "float", "short", "byte", "char", "boolean", "string", "list",
        "arraylist", "map", "hashmap", "set", "hashset", "deque", "queue", "stack", "object",
    ]
    .into_iter()
    .collect();
    let operators: &[char] = &[
        '+', '-', '*', '/', '%', '=', '>', '<', '!', '&', '|', '^', '~', '?', ':',
    ];
    let punctuations: &[char] = &['(', ')', '[', ']', '{', '}', '.', ',', ';'];

    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if *in_block_comment {
            let start = i;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    *in_block_comment = false;
                    break;
                }
                i += 1;
            }
            if *in_block_comment {
                i = chars.len();
            }
            tokens.push((TokenKind::Comment, chars[start..i].iter().collect()));
            continue;
        }

        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            tokens.push((TokenKind::Comment, chars[i..].iter().collect()));
            break;
        }
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            let start = i;
            i += 2;
            *in_block_comment = true;
            while i + 1 < chars.len() {
                if chars[i] == '*' && chars[i + 1] == '/' {
                    i += 2;
                    *in_block_comment = false;
                    break;
                }
                i += 1;
            }
            tokens.push((TokenKind::Comment, chars[start..i].iter().collect()));
            continue;
        }

        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            if i > chars.len() {
                i = chars.len();
            }
            tokens.push((TokenKind::String, chars[start..i].iter().collect()));
            continue;
        }

        if chars[i].is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            tokens.push((TokenKind::Number, chars[start..i].iter().collect()));
            continue;
        }

        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let lower = word.to_ascii_lowercase();

            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            let is_function = j < chars.len() && chars[j] == '(';
            let kind = if keywords.contains(lower.as_str()) {
                TokenKind::Keyword
            } else if type_words.contains(lower.as_str())
                || word.chars().next().is_some_and(|ch| ch.is_ascii_uppercase())
            {
                TokenKind::TypeName
            } else if is_function {
                TokenKind::Function
            } else {
                TokenKind::Default
            };

            tokens.push((kind, word));
            continue;
        }

        if operators.contains(&chars[i]) {
            tokens.push((TokenKind::Operator, chars[i].to_string()));
            i += 1;
            continue;
        }
        if punctuations.contains(&chars[i]) {
            tokens.push((TokenKind::Punctuation, chars[i].to_string()));
            i += 1;
            continue;
        }

        tokens.push((TokenKind::Default, chars[i].to_string()));
        i += 1;
    }
    tokens
}

#[cfg(windows)]
fn enable_vt_mode() {
    use winapi::shared::minwindef::DWORD;
    use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::winbase::STD_OUTPUT_HANDLE;
    use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode: DWORD = 0;
        if GetConsoleMode(h, &mut mode) != 0 {
            let _ = SetConsoleMode(h, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
        }
    }
}
