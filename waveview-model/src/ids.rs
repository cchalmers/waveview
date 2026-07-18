use serde::{Deserialize, Serialize};

/// Identity of a signal within a loaded waveform.
///
/// The numeric representation is deliberately opaque. Row positions are not identities: a signal
/// keeps this ID while displayed items are filtered or reordered.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SignalId(u64);

impl SignalId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Identity of an item in the viewer's displayed-item tree.
///
/// Multiple displayed items may eventually refer to the same [`SignalId`] while retaining separate
/// aliases, formats, heights, and group positions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct DisplayedItemId(u64);

impl DisplayedItemId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_and_displayed_item_ids_are_distinct_types() {
        let signal = SignalId::new(7);
        let displayed = DisplayedItemId::new(7);

        assert_eq!(signal.get(), displayed.get());
        assert_eq!(format!("{signal:?}"), "SignalId(7)");
        assert_eq!(format!("{displayed:?}"), "DisplayedItemId(7)");
    }
}
