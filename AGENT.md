# Project Rules

- CLI output must be in Chinese.
- Do not add comments to project files.
- Data must be local only; do not use web crawling or online fetching.
- Use key-value mapping from problem id in data/problems.json.
- Documentation and metadata must be in English.
- Every problem entry must include non-empty answer code (Java).
- For id lookup, users must pass -i/--hint, -a/--answer, or -e/--extra; at least one required.
- -o/--output and Markdown modes (-m/--md, -r/--render) are deprecated in favor of direct colorized CLI output.
- Use the short command name lh in examples; leetcode-helper remains supported as alternative.
- Default data comes from embedded dataset compiled from data/problems.json; --data can load external file.
- Output format: compact (section labels and content on same line, API notes in single line), no unnecessary blank lines.
- All Markdown syntax elements must be colored according to active theme from theme.toml.
- Code syntax highlighting applies to answer blocks: keywords, type names, strings, numbers, comments, operators, punctuation.

# Documentation update policy
- When code or behavior changes, ALL documentation files must be updated to reflect the change. Update at least: `README.md`, `README_zh.md`, `CHECKLIST.md`, and this `AGENT.md` with a short summary and date.
- Any change that affects CLI output format, flags, or theme behavior requires a documentation update before merging.

# Tech Stack and Versions
- Language: Rust (edition 2024), stable toolchain.
- CLI framework: clap v4 (derive).
- Serialization: serde v1 (derive), serde_json v1.
- Configuration: toml v0.8.
- Terminal colors: colored v2 (always enabled, theme-driven).
- Windows VT support: winapi v0.3 (consoleapi, wincon, processenv, winbase).
- Data format: local JSON file (data/problems.json).
- Testing: Rust built-in test framework (cargo test).

# Directory Structure and File Roles
- AGENT.md: project rules and documentation.
- README.md: English features, usage, and data format.
- README_zh.md: Chinese features, usage, theme configuration, output examples.
- CHECKLIST.md: tasks and milestones.
- theme.toml: color theme configuration (three sections: markdown, api, syntax).
- Cargo.toml: crate configuration and dependency versions; package name leetcode-helper; binaries lh (short) and leetcode-helper (full).
- data/: local dataset directory.
  - problems.json: dataset source; root key "problems" is an id-to-problem mapping.
    - Fields: id, title, category, solution, description, essence, analogy, container, steps (array), complexity, answer (required).
    - Optional: example, diagram, apiNotes (array of {api, usage, note}).
- src/: core source code.
  - main.rs: CLI entry; parses args (with --theme for custom theme), loads data, formats output with theme colors and compact layout.
  - lib.rs: data model (Problem, Database, ApiNote) and core logic (JSON parsing, id lookup, keyword search, sorting).
- tests/: integration tests.
  - cli_smoke.rs: CLI smoke test; verifies output includes expected titles.
