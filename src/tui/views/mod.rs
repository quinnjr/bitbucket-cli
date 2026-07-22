/// Available views in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Dashboard,
    Repositories,
    PullRequests,
    Issues,
    Pipelines,
}

/// State for list-based views
#[derive(Debug, Default)]
pub struct ViewState {
    /// Currently selected index
    pub selected_index: usize,
}

impl ViewState {
    /// Move selection up
    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn next(&mut self, max: usize) {
        if max > 0 && self.selected_index < max - 1 {
            self.selected_index += 1;
        }
    }
}
