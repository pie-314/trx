 feat/help-tab-hint

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
 dev
pub enum InputMode {
    Normal,
    Editing,
    DetailScrolling,   // <-- Add this
}