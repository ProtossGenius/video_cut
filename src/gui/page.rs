#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Navigation,
    MainInterface,
    Editor,
}
