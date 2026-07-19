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
