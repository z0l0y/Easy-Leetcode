# LeetCode Helper CLI

Read this in other languages: [English](README.md), [中文](README_zh.md)

**The missing debugger for LeetCode.** Auto-instruments your Java solutions, runs them against real test cases, and visualizes every step — variables, data structures, call stack — in an interactive terminal debugger. No browser, no copy-paste, no `System.out.println` ever again.

## Features

### 🔍 Knowledge base
- Instant lookup by problem number (`lh 94`) or keyword search (`lh -s binary tree`)
- Browse all problems at a glance (`lh -l`)
- Reveal hints, solutions, notes, and complexity analysis on demand (`-i` `-a` `-e`)

### 🐛 Algorithm debugger (the killer feature)
- **One command**: `lh 206 -t` instruments, compiles, and runs Java code locally — no browser ever
- **Step-through TUI**: `→`/`←` to walk execution line-by-line, watch every variable change on each step
- **Data structure diffing**: changed nodes marked `*` inline — see *what* and *where* at a glance
- **8 visualizations**: arrays, linked lists (with `↓cur` pointer annotations), binary trees (`/` `\` branches), hashmaps, stacks, queues, sliding windows, DP heatmaps
- **Breakpoints**: `b` to set, `c` to run-to-breakpoint, just like gdb
- **Variable search**: `/` to find and highlight variables by name
- **Call stack panel**: tracks recursive calls — `dfs(left) → dfs(left) → dfs(left)`
- **Instant replay**: cached traces replay instantly — re-run on a fresh input only when you want

### 🎨 Polish
- Color themes via `theme.toml` with hex `#RRGGBB` support for TUI colors
- Global config via `~/.lhconfig.toml` (default theme, cache dir, JDK path)
- Shell completions for bash / zsh / fish / powershell
- Pipe-safe: auto-detects non-TTY output and strips ANSI codes
- Statically linked binary, no runtime dependencies beyond a JDK

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
- TUI keyboard shortcuts:
  - **Step:** `Enter` / `→` / `Space` / `j` = next, `←` / `k` = prev, `g` = first, `G` = last
  - **Breakpoints:** `b` = toggle breakpoint at current line, `c` = continue to next breakpoint
  - **Search:** `/` = search mode (type then Enter to confirm, Esc to cancel)
  - **Scroll:** `PgUp`/`K` = scroll up, `PgDn`/`J` = scroll down, `↑`/`↓` = scroll 1 line
  - **Quit:** `q` / `Esc`
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

### 5. TUI Interface Colors
```toml
[tui]
line_no = "#808080"          # Line number color
cur_line_bg = "#282c34"      # Current line background
cur_marker = "#61afef"       # Current line arrow (▶)
var_name = "#61afef"         # Variable name
var_value = "#e5c07b"        # Variable value
changed = "#ffc850"          # Changed variable highlight
changed_bg = "#3c3200"       # Changed variable background
title = "#98c379"            # Panel titles
border = "#5c6370"           # Panel borders
status = "#abb2bf"           # Status bar text
status_bar_bg = "#21252b"    # Status bar background
source_bar_bg = "#21252b"    # Source bar background
result = "#98c379"           # Result highlight
```
TUI colors support hex `#RRGGBB` format in addition to named colors.

## Global Config (`.lhconfig.toml`)

Place `.lhconfig.toml` in the current directory, any parent directory, or your home directory:

```toml
# JDK installation path (optional)
jdk_path = "/usr/lib/jvm/java-17"

# Trace cache directory (overrides data/traces/)
cache_dir = "/home/user/.cache/lh"

# Default theme file
default_theme = "my-theme.toml"

# Default trace mode: "tui" or "text"
trace_mode = "tui"
```

CLI arguments always take precedence over config file values.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `LH_CACHE_DIR` | Trace cache directory (overrides config `cache_dir`) |

## Shell Completions

Generate completion scripts for your shell:

```bash
lh completions bash  > ~/.bash_completion.d/lh
lh completions zsh   > ~/.zfunc/_lh
lh completions fish  > ~/.config/fish/completions/lh.fish
lh completions powershell  # prints PowerShell completion
```

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
