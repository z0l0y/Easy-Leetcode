use crate::SyntaxTheme;
use colored::Colorize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
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

pub fn highlight_code_line(
    line: &str,
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

pub fn lex_code_line(line: &str, in_block_comment: &mut bool) -> Vec<(TokenKind, String)> {
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

/// Truncate a value string for display, limiting to `max_len` chars.
/// Long arrays get abbreviated: [1, 2, 3, ..., 100] → [1, 2, 3, … (+97 more)]
pub fn truncate_value(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() > 8 {
            let shown: Vec<&str> = parts.iter().take(8).map(|p| p.trim()).collect();
            return format!("[{} … (+{} more)]", shown.join(", "), parts.len() - 8);
        }
    }
    if s.len() > max_len {
        let trunc = s.chars().take(max_len - 1).collect::<String>();
        return format!("{}…", trunc);
    }
    s.to_string()
}
