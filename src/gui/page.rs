#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Navigation,
    MainInterface,
    FileBrowser,
    Editor,
}

impl Default for Page {
    fn default() -> Self {
        Self::Navigation
    }
}
