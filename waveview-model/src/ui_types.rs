//! Stable intent types shared across the native host and reloadable UI.
//!
//! Changing these types requires restarting a running reload session.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    OpenFile,
    OpenUrl,
    ConnectLive,
    Reset,
    Quit,
    UndoDisplayChange,
    RedoDisplayChange,
    RemoveFocusedItem,
    ToggleSignalBrowser,
    FitTime,
    ShowInfo,
    ShowSamples,
    HideInspector,
    ShowKeyHelp,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PromptOutputKind {
    #[default]
    Normal,
    Command,
    Result,
    Error,
    Muted,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PromptOutput {
    pub text: String,
    pub kind: PromptOutputKind,
}

impl PromptOutput {
    pub fn new(text: impl Into<String>, kind: PromptOutputKind) -> Self {
        Self {
            text: text.into(),
            kind,
        }
    }
}
