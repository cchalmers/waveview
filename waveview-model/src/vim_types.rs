#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VimInput {
    Char(char),
    Ctrl(char),
    Escape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VimMode {
    Normal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RepeatableChange {
    MoveFocused(isize),
    RemoveFocused(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VimState {
    pub(crate) mode: VimMode,
    pub(crate) count: Option<usize>,
    pub(crate) pending: String,
    pub(crate) last_change: Option<RepeatableChange>,
    pub(crate) repeating_zoom: Option<bool>,
}

impl Default for VimState {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            count: None,
            pending: String::new(),
            last_change: None,
            repeating_zoom: None,
        }
    }
}
