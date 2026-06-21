# LeetCode Helper CLI

Read this in other languages: [English](README.md), [中文](README_zh.md)

A Rust CLI for browsing LeetCode Hot100 notes by id or keyword. Features a built-in algorithm execution visualizer with TUI step-through debugging.

## Features
- Lookup by id
- Keyword search
- List all problems
- Show hint/answer/extra sections on demand (`-i` / `-a` / `-e`)
- **Algorithm execution trace** — truly runs Java code, captures real variable values at each step
- **Interactive TUI mode** — step through code line-by-line, watch variables change (like an IDE debugger)
- Auto-instrumentation (zero AI, zero pre-written trace data) with instant cache replay
- Custom input parameters (`--input`) for testing different test cases
- Colorful terminal rendering with theme support (Markdown, Java syntax highlighting)
- Customizable color theme via `theme.toml`
- Embedded local dataset (`data/problems.json` compiled into binary)

## Usage
```bash
# Core queries
lh 76 -i                      # show hint
lh 76 -a                      # show answer code
lh 76 -e                      # show extra (example, diagram, API notes)
lh 76 -i -a -e                # show all sections

# Algorithm trace (recommended)
lh 1 -t                       # TUI interactive mode: step through code, watch variables
lh 53 -t                      # Kadane's algorithm, 28-step trace
lh 1 -t --input "nums=[1,2,3,4,5], target=9"  # custom input

# Other trace options
lh 1 -t --trace-text          # plain text output (single dump)
lh 1 -t --re-trace            # force regenerate (skip cache)

# General
lh -l                         # list all problems
lh -s window                  # keyword search
lh 76 -e --theme my-theme.toml  # use custom theme
```

Available flags:
| Flag | Description |
|------|-------------|
| `-i, --hint` | Show hint section |
| `-a, --answer` | Show answer code |
| `-t, --trace` | **Algorithm trace** — launches interactive TUI (default) |
| `--trace-text` | Plain text trace output (use with `-t`) |
| `--input <INPUT>` | Custom input, e.g. `"nums=[1,2], target=3"` |
| `--re-trace` | Force re-run auto-trace (skip cache & static data) |
| `-e, --extra` | Show extra section (example, diagram, API notes) |
| `-l, --list` | List all problems |
| `-s, --search` | Treat query as keyword search |
| `--theme <FILE>` | Use custom theme file, defaults to `theme.toml` |

Notes:
- For id lookup, at least one of `-i`, `-a`, `-e`, `-t` is required.
- **Execution trace** (`-t`) compiles and runs the problem's Java solution, capturing variable values at every step. First run takes 1-3 seconds; results are cached to `data/traces/` for instant replay.
- TUI keyboard shortcuts: `Enter`/`→` next step, `←` previous step, `g` first step, `G` last step, `q` quit.
- Use `--input` to specify custom parameters; step count scales with input size automatically.
- Color output is enabled by default.

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

### 4. Trace Visualization Colors
```toml
[trace]
header = "bright_green"      # "Execution Trace:" label
step_number = "bright_cyan"  # "Step 1/28"
separator = "bright_black"   # Separator lines
arrow = "bright_green"       # "→" prefix
code_line = "bright_white"   # Code line text
var_name = "bright_blue"     # Variable name
var_value = "bright_white"   # Variable value
var_old = "bright_black"     # Old value "(was: ...)"
note = "bright_black"        # Step notes
ds_label = "bright_magenta"  # Data structure label
ds_highlight = "bright_yellow" # Highlighted elements
ds_pointer = "bright_green"  # Pointer markers (^L, ^R)
result = "bright_green"      # Result highlight / return value
loop_back = "bright_black"   # "[loop]" marker
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
