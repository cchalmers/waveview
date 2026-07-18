use serde::{Deserialize, Serialize};

use crate::{vcd, SignalId};

/// One named signal in a loaded capture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WaveformSignal {
    id: SignalId,
    name: String,
    signal: vcd::Signal,
}

impl WaveformSignal {
    pub fn id(&self) -> SignalId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn signal(&self) -> &vcd::Signal {
        &self.signal
    }
}

/// Immutable capture data. Display order and filtering live in [`crate::viewer::ViewerState`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Waveform {
    signals: Vec<WaveformSignal>,
    end_time: u64,
}

impl Waveform {
    pub fn new(signals: Vec<(String, vcd::Signal)>, end_time: u64) -> Self {
        let signals = signals
            .into_iter()
            .enumerate()
            .map(|(index, (name, signal))| WaveformSignal {
                id: SignalId::new(index as u64),
                name,
                signal,
            })
            .collect();
        Self { signals, end_time }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), 1)
    }

    pub fn signals(&self) -> &[WaveformSignal] {
        &self.signals
    }

    pub fn signal(&self, id: SignalId) -> Option<&WaveformSignal> {
        self.signals.iter().find(|signal| signal.id == id)
    }

    pub fn end_time(&self) -> u64 {
        self.end_time
    }
}

impl Default for Waveform {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_stable_ids_within_a_capture() {
        let waveform = Waveform::new(
            vec![
                ("top.a".to_owned(), vcd::Signal::new(1)),
                ("top.b".to_owned(), vcd::Signal::new(1)),
            ],
            50,
        );

        assert_eq!(waveform.signals()[0].id(), SignalId::new(0));
        assert_eq!(waveform.signals()[1].id(), SignalId::new(1));
        assert_eq!(waveform.signal(SignalId::new(1)).unwrap().name(), "top.b");
        assert_eq!(waveform.end_time(), 50);
    }
}
