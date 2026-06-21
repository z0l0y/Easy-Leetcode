use leetcode_helper::{Trace, TraceStep};

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
    /// Set to true to exit the TUI
    pub should_quit: bool,
}

impl<'a> AppState<'a> {
    pub fn new(
        trace: &'a Trace,
        code_lines: &'a [String],
        method_start: usize,
        method_end: usize,
    ) -> Self {
        Self {
            trace,
            code_lines,
            method_start,
            method_end,
            current_step: 0,
            scroll_offset: 0,
            should_quit: false,
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
        }
    }

    pub fn prev_step(&mut self) {
        self.current_step = self.current_step.saturating_sub(1);
    }

    pub fn first_step(&mut self) {
        self.current_step = 0;
    }

    pub fn last_step(&mut self) {
        self.current_step = self.trace.steps.len().saturating_sub(1);
    }

    /// Auto-scroll the code panel to keep the current line visible.
    pub fn autoscroll(&mut self, visible_lines: usize) {
        let line_idx = self
            .current_line()
            .saturating_sub(self.method_start); // 0-indexed within method

        // Make sure current line is in [scroll_offset, scroll_offset + visible_lines)
        if line_idx < self.scroll_offset {
            self.scroll_offset = line_idx;
        } else if line_idx >= self.scroll_offset + visible_lines.saturating_sub(2) {
            self.scroll_offset = line_idx.saturating_sub(visible_lines.saturating_sub(3));
        }
    }

    /// Visible code lines for the code panel.
    pub fn visible_code_slice(&self) -> &[String] {
        let start = self.method_start.saturating_sub(1); // 0-indexed
        let end = self.method_end.min(self.code_lines.len());
        if start <= end && start < self.code_lines.len() {
            &self.code_lines[start..end]
        } else {
            &[]
        }
    }
}
