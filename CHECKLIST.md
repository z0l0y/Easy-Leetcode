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
- [x] Fix spacing: no blank lines between section label and content (e.g., "解法:" → immediately next line is content)
- [x] Zero compilation warnings
- [x] Update all documentation (README_zh.md, README.md, AGENT.md)

## Known Limitations
- Answer code is assumed to be Java; syntax highlighting rules apply to Java only (no language auto-detect)
- Theme colors are limited to ANSI 16-color palette
- Windows console requires VT100 mode enabled (via winapi calls)
- No multi-line API descriptions (single-line format only)
