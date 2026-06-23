use crate::TuiTheme;
use leetcode_helper::{Trace, TraceStep};
use std::collections::HashSet;

/// TUI application state for the trace viewer.
pub struct AppState<'a> {
    /// The execution trace data
    pub trace: &'a Trace,
    /// Full answer code lines (for display)
    pub code_lines: &'a [String],
    /// 1-indexed line number where the method body starts
    pub method_start: usize,
    /// 1-indexed line number where the method body ends
    pub method_end: usize,
    /// Current step index (0-based into trace.steps)
    pub current_step: usize,
    /// Vertical scroll offset for the code panel
    pub scroll_offset: usize,
    /// Number of visible lines in the code panel (set by render, used by autoscroll)
    pub visible_lines: usize,
    /// TUI color theme from theme.toml [tui] section
    pub tui_theme: &'a TuiTheme,
    /// Breakpoints: set of 1-indexed line numbers
    pub breakpoints: HashSet<usize>,
    /// Whether we're in "continue to breakpoint" mode
    pub running: bool,
    /// Search mode active
    pub search_mode: bool,
    /// Search query string
    pub search_query: String,
    /// Whether search query has been confirmed
    pub search_active: bool,
    /// Set to true to exit the TUI
    pub should_quit: bool,
    /// Pre-rendered DS body lines for fast TUI rendering: [step_idx][ds_idx] → lines
    pub ds_render_cache: Vec<Vec<Vec<String>>>,
}

impl<'a> AppState<'a> {
    pub fn new(
        trace: &'a Trace,
        code_lines: &'a [String],
        method_start: usize,
        method_end: usize,
        tui_theme: &'a TuiTheme,
    ) -> Self {
        Self {
            trace,
            code_lines,
            method_start,
            method_end,
            current_step: 0,
            scroll_offset: 0,
            visible_lines: 20,
            tui_theme,
            breakpoints: HashSet::new(),
            running: false,
            search_mode: false,
            search_query: String::new(),
            search_active: false,
            should_quit: false,
            ds_render_cache: Vec::new(),
        }
    }

    /// Get the current trace step, if any.
    pub fn current(&self) -> Option<&TraceStep> {
        self.trace.steps.get(self.current_step)
    }

    pub fn total_steps(&self) -> usize {
        self.trace.steps.len()
    }

    /// Current line number (1-indexed) from the trace step, or method_start.
    pub fn current_line(&self) -> usize {
        self.current()
            .and_then(|s| s.line.parse::<usize>().ok())
            .unwrap_or(self.method_start)
    }

    pub fn next_step(&mut self) {
        if self.current_step + 1 < self.trace.steps.len() {
            self.current_step += 1;
            self.autoscroll();
        }
    }

    pub fn prev_step(&mut self) {
        if self.current_step > 0 {
            self.current_step -= 1;
            self.autoscroll();
        }
    }

    pub fn first_step(&mut self) {
        self.current_step = 0;
        self.autoscroll();
    }

    pub fn last_step(&mut self) {
        self.current_step = self.trace.steps.len().saturating_sub(1);
        self.autoscroll();
    }

    /// Toggle a breakpoint at the current line.
    pub fn toggle_breakpoint(&mut self) {
        let line = self.current_line();
        if self.breakpoints.contains(&line) {
            self.breakpoints.remove(&line);
        } else {
            self.breakpoints.insert(line);
        }
    }

    /// Continue running until a breakpoint or end of trace.
    pub fn continue_to_breakpoint(&mut self) {
        self.running = true;
        // Will be handled in the event loop — step until breakpoint or end
    }

    /// Advance one step during "running" mode. Returns true if should continue.
    pub fn running_step(&mut self) -> bool {
        if self.current_step + 1 >= self.trace.steps.len() {
            self.running = false;
            return false;
        }
        self.current_step += 1;
        self.autoscroll();
        if self.breakpoints.contains(&self.current_line()) {
            self.running = false;
            return false;
        }
        true
    }

    /// Auto-scroll the code panel to keep the current line visible.
    pub fn autoscroll(&mut self) {
        let line_idx = self
            .current_line()
            .saturating_sub(self.method_start);
        let vis = self.visible_lines.max(1);

        if line_idx < self.scroll_offset {
            self.scroll_offset = line_idx;
        } else if line_idx >= self.scroll_offset + vis.saturating_sub(2) {
            self.scroll_offset = line_idx.saturating_sub(vis.saturating_sub(3));
        }
    }

    /// Visible code lines for the code panel.
    pub fn visible_code_slice(&self) -> &[String] {
        let start = self.method_start.saturating_sub(1);
        let end = self.method_end.min(self.code_lines.len());
        if start <= end && start < self.code_lines.len() {
            &self.code_lines[start..end]
        } else {
            &[]
        }
    }
}
