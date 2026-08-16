#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Navigation,
    MainInterface,
    Editor,
}

impl Default for Page {
    fn default() -> Self {
        Self::Navigation
    }
}
