use std::process::Command;

#[derive(Debug, Clone)]
pub struct LogsState {
    pub lines: Vec<String>,
    pub scroll_offset: usize,
}

impl Default for LogsState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsState {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn load(&mut self) {
        let output = Command::new("journalctl")
            .args(["-p", "err", "-n", "100", "--no-pager", "-q"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let lines: Vec<String> = stdout
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| l.to_string())
                    .collect();

                self.lines = lines;
                self.scroll_offset = 0;
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                self.lines = vec![format!("journalctl error: {err}")];
                self.scroll_offset = 0;
            }
            Err(e) => {
                self.lines = vec![format!("Failed to execute journalctl: {e}")];
                self.scroll_offset = 0;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, visible_lines: usize) {
        if self.lines.len() > visible_lines {
            let max_offset = self.lines.len().saturating_sub(visible_lines);
            if self.scroll_offset < max_offset {
                self.scroll_offset += 1;
            }
        }
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, page_size: usize, visible_lines: usize) {
        if self.lines.len() > visible_lines {
            let max_offset = self.lines.len().saturating_sub(visible_lines);
            self.scroll_offset = (self.scroll_offset + page_size).min(max_offset);
        }
    }

    pub fn scroll_home(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_end(&mut self, visible_lines: usize) {
        if self.lines.len() > visible_lines {
            self.scroll_offset = self.lines.len().saturating_sub(visible_lines);
        }
    }
}
