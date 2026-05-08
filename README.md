# LeetCode Helper CLI

Read this in other languages: [English](README.md), [中文](README_zh.md)

A Rust CLI for browsing Chinese Hot100 notes by id or keyword.

## Features
- Lookup by id
- Keyword search
- List all problems
- Show hint/answer/extra sections on demand (`-i` / `-a` / `-e`)
- Colorful terminal rendering with theme support (Markdown inline syntax, code syntax highlighting)
- Customizable color theme via `theme.toml`
- Compact output format (section labels and content on same line where applicable)
- Embedded local dataset (`data/problems.json` compiled into binary)

## Usage
```bash
lh 76 -i                      # show hint
lh 76 -a                      # show answer code
lh 76 -e                      # show extra (example, diagram, API notes)
lh 76 -i -a -e                # show all sections
lh -l                         # list all problems
lh -s window                  # keyword search
lh 76 -e --theme my-theme.toml  # use custom theme
```

Available flags:
- `-i, --hint` show hint section
- `-a, --answer` show answer code
- `-e, --extra` show extra section (example, diagram, API notes)
- `-l, --list` list all problems
- `-s, --search` treat query as keyword search
- `--theme <FILE>` use theme file, defaults to `theme.toml` in project root

Notes:
- For id lookup, at least one of `-i`, `-a`, `-e` is required.
- Output is always rendered directly in terminal with colors enabled by default.
- Answer code is highlighted using Java syntax highlighting rules (keywords, comments, strings, etc.).
- All Markdown syntax elements are colored according to the active theme.

## Theme Configuration

Default configuration: see `theme.toml`. Three configuration sections available:

### 1. Markdown Elements
```toml
[markdown]
title = "bright_yellow"          # Main title (problem id and number)
section_label = "bright_green"   # Section labels (description, container, etc.)
code_block = "bright_cyan"       # Code blocks
inline_code = "cyan"             # Inline code (`code`)
bold = "bright_white"            # Bold text (**text**)
link = "bright_blue"             # Links
blockquote = "bright_black"      # Blockquotes (> text)
h1 = "bright_yellow"             # Level 1 heading
h2 = "bright_yellow"             # Level 2 heading
h3 = "bright_white"              # Level 3 heading
list_marker = "green"            # List item prefix
```

### 2. API Notes Colors
```toml
[api]
api_name = "bright_magenta"   # API method name
usage_label = "cyan"          # "usage:" label
note_label = "yellow"         # "note:" label
```

### 3. Code Syntax Highlighting
```toml
[syntax]
default = "bright_white"      # Default text
keyword = "bright_yellow"     # Keywords (if/for/while/etc.)
type_name = "bright_blue"     # Type names (int/String/HashMap/etc.)
function = "bright_cyan"      # Function calls (foo())
string = "bright_magenta"     # String literals
number = "bright_red"         # Numbers
comment = "green"             # // and /* */ comments
operator = "bright_white"     # Operators (+/-/* /)
punctuation = "bright_black"  # Brackets, semicolons
```

Supported colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`.

## Build and Install
Local run:

```bash
cargo run --bin lh -- 76 -i
```

Release build:

```bash
cargo build --release
```

## Data Format
Root key: `problems` (JSON object mapping id to problem).

Each problem includes:
- `id`, `title`, `category`, `solution`
- `description`, `essence`, `analogy`, `container`, `steps` (array), `complexity`
- `answer` (Java code, required and non-empty)
- Optional: `example`, `diagram`, `apiNotes` (array of {api, usage, note})

## Development
```bash
cargo test
```
