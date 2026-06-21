use colored::Color;
use ratatui::style::Color as TuiColor;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Clone, Deserialize)]
struct ThemeToml {
    syntax: Option<SyntaxThemeToml>,
    markdown: Option<MarkdownThemeToml>,
    api: Option<ApiThemeToml>,
    trace: Option<TraceThemeToml>,
    tui: Option<TuiThemeToml>,
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

#[derive(Clone, Deserialize)]
struct TraceThemeToml {
    header: Option<String>,
    step_number: Option<String>,
    separator: Option<String>,
    arrow: Option<String>,
    code_line: Option<String>,
    var_name: Option<String>,
    var_value: Option<String>,
    var_old: Option<String>,
    note: Option<String>,
    ds_label: Option<String>,
    ds_highlight: Option<String>,
    ds_pointer: Option<String>,
    result: Option<String>,
    loop_back: Option<String>,
}

#[derive(Clone, Deserialize)]
struct TuiThemeToml {
    line_no: Option<String>,
    cur_line_bg: Option<String>,
    cur_marker: Option<String>,
    var_name: Option<String>,
    var_value: Option<String>,
    changed: Option<String>,
    changed_bg: Option<String>,
    title: Option<String>,
    border: Option<String>,
    status: Option<String>,
    status_bar_bg: Option<String>,
    source_bar_bg: Option<String>,
    result: Option<String>,
}

#[derive(Clone)]
pub(crate) struct SyntaxTheme {
    pub default: Color,
    pub keyword: Color,
    pub type_name: Color,
    pub function: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub operator: Color,
    pub punctuation: Color,
}

#[derive(Clone)]
pub(crate) struct MarkdownTheme {
    pub title: Color,
    pub section_label: Color,
    pub code_block: Color,
    pub inline_code: Color,
    pub bold: Color,
    pub link: Color,
    pub blockquote: Color,
    pub h1: Color,
    pub h2: Color,
    pub h3: Color,
    pub list_marker: Color,
}

#[derive(Clone)]
pub(crate) struct ApiTheme {
    pub api_name: Color,
    pub usage_label: Color,
    pub note_label: Color,
}

#[derive(Clone)]
pub(crate) struct TraceTheme {
    pub header: Color,
    pub step_number: Color,
    pub separator: Color,
    pub arrow: Color,
    pub code_line: Color,
    pub var_name: Color,
    pub var_value: Color,
    pub var_old: Color,
    pub note: Color,
    pub ds_label: Color,
    pub ds_highlight: Color,
    pub ds_pointer: Color,
    pub result: Color,
    pub loop_back: Color,
}

#[derive(Clone)]
pub(crate) struct TuiTheme {
    pub line_no: TuiColor,
    pub cur_line_bg: TuiColor,
    pub cur_marker: TuiColor,
    pub var_name: TuiColor,
    pub var_value: TuiColor,
    pub changed: TuiColor,
    pub changed_bg: TuiColor,
    pub title: TuiColor,
    pub border: TuiColor,
    pub status: TuiColor,
    pub status_bar_bg: TuiColor,
    pub source_bar_bg: TuiColor,
    pub result: TuiColor,
}

#[derive(Clone)]
pub(crate) struct Theme {
    pub syntax: SyntaxTheme,
    pub markdown: MarkdownTheme,
    pub api: ApiTheme,
    pub trace: TraceTheme,
    pub tui: TuiTheme,
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

impl Default for TraceTheme {
    fn default() -> Self {
        Self {
            header: Color::BrightGreen,
            step_number: Color::BrightCyan,
            separator: Color::BrightBlack,
            arrow: Color::BrightGreen,
            code_line: Color::BrightWhite,
            var_name: Color::BrightBlue,
            var_value: Color::BrightWhite,
            var_old: Color::BrightBlack,
            note: Color::BrightBlack,
            ds_label: Color::BrightMagenta,
            ds_highlight: Color::BrightYellow,
            ds_pointer: Color::BrightGreen,
            result: Color::BrightGreen,
            loop_back: Color::BrightBlack,
        }
    }
}

impl Default for TuiTheme {
    fn default() -> Self {
        Self {
            line_no: TuiColor::Gray,
            cur_line_bg: TuiColor::Rgb(40, 44, 52),
            cur_marker: TuiColor::Rgb(97, 175, 239),
            var_name: TuiColor::Rgb(97, 175, 239),
            var_value: TuiColor::Rgb(229, 192, 123),
            changed: TuiColor::Rgb(255, 200, 80),
            changed_bg: TuiColor::Rgb(60, 50, 0),
            title: TuiColor::Rgb(152, 195, 121),
            border: TuiColor::Rgb(92, 99, 112),
            status: TuiColor::Rgb(171, 178, 191),
            status_bar_bg: TuiColor::Rgb(33, 37, 43),
            source_bar_bg: TuiColor::Rgb(33, 37, 43),
            result: TuiColor::Rgb(152, 195, 121),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            syntax: SyntaxTheme::default(),
            markdown: MarkdownTheme::default(),
            api: ApiTheme::default(),
            trace: TraceTheme::default(),
            tui: TuiTheme::default(),
        }
    }
}

pub(crate) fn load_theme(theme_path: Option<&str>) -> Theme {
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

    if let Some(trace) = config.trace {
        if let Some(value) = trace.header.and_then(|s| parse_color_name(&s)) {
            theme.trace.header = value;
        }
        if let Some(value) = trace.step_number.and_then(|s| parse_color_name(&s)) {
            theme.trace.step_number = value;
        }
        if let Some(value) = trace.separator.and_then(|s| parse_color_name(&s)) {
            theme.trace.separator = value;
        }
        if let Some(value) = trace.arrow.and_then(|s| parse_color_name(&s)) {
            theme.trace.arrow = value;
        }
        if let Some(value) = trace.code_line.and_then(|s| parse_color_name(&s)) {
            theme.trace.code_line = value;
        }
        if let Some(value) = trace.var_name.and_then(|s| parse_color_name(&s)) {
            theme.trace.var_name = value;
        }
        if let Some(value) = trace.var_value.and_then(|s| parse_color_name(&s)) {
            theme.trace.var_value = value;
        }
        if let Some(value) = trace.var_old.and_then(|s| parse_color_name(&s)) {
            theme.trace.var_old = value;
        }
        if let Some(value) = trace.note.and_then(|s| parse_color_name(&s)) {
            theme.trace.note = value;
        }
        if let Some(value) = trace.ds_label.and_then(|s| parse_color_name(&s)) {
            theme.trace.ds_label = value;
        }
        if let Some(value) = trace.ds_highlight.and_then(|s| parse_color_name(&s)) {
            theme.trace.ds_highlight = value;
        }
        if let Some(value) = trace.ds_pointer.and_then(|s| parse_color_name(&s)) {
            theme.trace.ds_pointer = value;
        }
        if let Some(value) = trace.result.and_then(|s| parse_color_name(&s)) {
            theme.trace.result = value;
        }
        if let Some(value) = trace.loop_back.and_then(|s| parse_color_name(&s)) {
            theme.trace.loop_back = value;
        }
    }

    if let Some(tui) = config.tui {
        if let Some(value) = tui.line_no.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.line_no = value;
        }
        if let Some(value) = tui.cur_line_bg.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.cur_line_bg = value;
        }
        if let Some(value) = tui.cur_marker.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.cur_marker = value;
        }
        if let Some(value) = tui.var_name.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.var_name = value;
        }
        if let Some(value) = tui.var_value.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.var_value = value;
        }
        if let Some(value) = tui.changed.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.changed = value;
        }
        if let Some(value) = tui.changed_bg.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.changed_bg = value;
        }
        if let Some(value) = tui.title.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.title = value;
        }
        if let Some(value) = tui.border.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.border = value;
        }
        if let Some(value) = tui.status.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.status = value;
        }
        if let Some(value) = tui.status_bar_bg.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.status_bar_bg = value;
        }
        if let Some(value) = tui.source_bar_bg.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.source_bar_bg = value;
        }
        if let Some(value) = tui.result.and_then(|s| parse_tui_color_name(&s)) {
            theme.tui.result = value;
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

fn parse_tui_color_name(name: &str) -> Option<TuiColor> {
    match name.to_ascii_lowercase().as_str() {
        "black" => Some(TuiColor::Black),
        "red" => Some(TuiColor::Red),
        "green" => Some(TuiColor::Green),
        "yellow" => Some(TuiColor::Yellow),
        "blue" => Some(TuiColor::Blue),
        "magenta" => Some(TuiColor::Magenta),
        "cyan" => Some(TuiColor::Cyan),
        "white" => Some(TuiColor::White),
        "gray" | "grey" => Some(TuiColor::Gray),
        "bright_black" | "brightblack" => Some(TuiColor::Black),
        "bright_red" | "brightred" => Some(TuiColor::Red),
        "bright_green" | "brightgreen" => Some(TuiColor::Green),
        "bright_yellow" | "brightyellow" => Some(TuiColor::Yellow),
        "bright_blue" | "brightblue" => Some(TuiColor::Blue),
        "bright_magenta" | "brightmagenta" => Some(TuiColor::Magenta),
        "bright_cyan" | "brightcyan" => Some(TuiColor::Cyan),
        "bright_white" | "brightwhite" => Some(TuiColor::White),
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).ok()?;
            let g = u8::from_str_radix(&s[3..5], 16).ok()?;
            let b = u8::from_str_radix(&s[5..7], 16).ok()?;
            Some(TuiColor::Rgb(r, g, b))
        }
        _ => None,
    }
}
