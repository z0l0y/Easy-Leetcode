# Checklist

## Phase 1: Visual Improvements
- [x] Add bolding and color highlighting for section titles and key content
- [x] Solve Markdown rendering in CLI (no native rendering; use colored crate + manual parsing)
- [x] Implement basic section formatting (label + content layout)

## Phase 2: Multi-Color System
- [x] Create configurable color theme system (theme.toml with [markdown], [syntax], [api] sections)
- [x] Implement Markdown syntax detection (bold **, code `...`, links [...](...), headers #, blockquotes >)
- [x] Add Java code syntax highlighting (keywords, strings, comments, numbers, operators, types)
- [x] Make theme colors distinct (not all blue; green for comments, yellow for keywords, etc.)
- [x] Support Windows terminal ANSI colors (VT100 mode via winapi)
- [x] Support loading theme from file (theme.toml default; --theme flag for custom)

## Phase 3: Compact Output and Polish
- [x] Remove output modes (-m/--md, -r/--render, -o/--output) in favor of direct colorized CLI
- [x] Implement compact output format (section labels and content on same line when applicable)
- [x] Implement API notes on single line ("- API名 用法: ... 说明: ...")
- [x] Remove unused functions (append_section)
- [x] Remove trailing newlines from render functions (let callers control spacing)
- [x] Fix spacing: only `description` and `container` collapse repeated blank lines; other fields keep original spacing
- [x] Zero compilation warnings
- [x] Update all documentation (README_zh.md, README.md, AGENT.md)

## Phase 4: Algorithm Execution Trace
- [x] Add Trace data model (Trace, TraceStep, TraceVar, TraceDs) to lib.rs
- [x] Add TraceTheme with 13 color keys to theme.toml and main.rs
- [x] Create src/trace.rs rendering engine (350+ lines)
- [x] Implement ASCII visualizations: array, hashmap, stack, queue, linkedlist, window
- [x] Add -t/--trace CLI flag and integration in main.rs
- [x] Add trace data for problem 1 (Two Sum, 9 steps)
- [x] Add trace data for problem 35 (Binary Search, 11 steps)
- [x] Add trace data for problem 3 (Sliding Window, 23 steps)
- [x] Update all documentation (README_zh.md, AGENT.md, CHECKLIST.md)

## Known Limitations
- Answer code is assumed to be Java; syntax highlighting rules apply to Java only (no language auto-detect)
- Theme colors are limited to ANSI 16-color palette
- Windows console requires VT100 mode enabled (via winapi calls)
- No multi-line API descriptions (single-line format only)
- Trace data only available for 3 problems (1, 3, 35); others show "暂无数据" message
- ASCII pointer alignment uses plain-text position calculation to work around ANSI codes
- Trace data must be hand-crafted per problem (no automatic code instrumentation)
