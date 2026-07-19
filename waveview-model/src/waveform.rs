use serde::{Deserialize, Serialize};

use crate::{vcd, SignalId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScopeKind {
    Module,
    Task,
    Function,
    Begin,
    Fork,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum VariableKind {
    Wire,
    Register,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SignalIndex {
    Bit(i32),
    Range(i32, i32),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScopeMetadata {
    name: String,
    kind: ScopeKind,
}

impl ScopeMetadata {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> ScopeKind {
        self.kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignalMetadata {
    scopes: Vec<ScopeMetadata>,
    reference: String,
    kind: VariableKind,
    width: u32,
    index: Option<SignalIndex>,
}

impl SignalMetadata {
    pub fn scopes(&self) -> &[ScopeMetadata] {
        &self.scopes
    }

    pub fn reference(&self) -> &str {
        &self.reference
    }

    pub fn kind(&self) -> VariableKind {
        self.kind
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn index(&self) -> Option<SignalIndex> {
        self.index
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScopeNode {
    metadata: ScopeMetadata,
    scopes: Vec<ScopeNode>,
    signals: Vec<SignalId>,
}

impl ScopeNode {
    pub fn metadata(&self) -> &ScopeMetadata {
        &self.metadata
    }

    pub fn scopes(&self) -> &[ScopeNode] {
        &self.scopes
    }

    pub fn signals(&self) -> &[SignalId] {
        &self.signals
    }

    pub fn signal_ids_recursive(&self) -> Vec<SignalId> {
        let mut ids = self.signals.clone();
        for scope in &self.scopes {
            ids.extend(scope.signal_ids_recursive());
        }
        ids
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignalHierarchy {
    scopes: Vec<ScopeNode>,
    signals: Vec<SignalId>,
}

impl SignalHierarchy {
    pub fn scopes(&self) -> &[ScopeNode] {
        &self.scopes
    }

    pub fn signals(&self) -> &[SignalId] {
        &self.signals
    }
}

/// One named signal in a loaded capture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WaveformSignal {
    id: SignalId,
    name: String,
    metadata: SignalMetadata,
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

    pub fn metadata(&self) -> &SignalMetadata {
        &self.metadata
    }
}

/// Immutable capture data. Display order and filtering live in [`crate::viewer::ViewerState`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Waveform {
    signals: Vec<WaveformSignal>,
    hierarchy: SignalHierarchy,
    end_time: u64,
}

impl Waveform {
    pub fn new(signals: Vec<(String, vcd::Signal)>, end_time: u64) -> Self {
        let signals = signals
            .into_iter()
            .enumerate()
            .map(|(signal_index, (name, signal))| {
                let mut parts = name.split('.').map(str::to_owned).collect::<Vec<_>>();
                let reference = parts.pop().unwrap_or_default();
                let scopes = parts
                    .into_iter()
                    .map(|name| ScopeMetadata {
                        name,
                        kind: ScopeKind::Module,
                    })
                    .collect();
                WaveformSignal {
                    id: SignalId::new(signal_index as u64),
                    metadata: SignalMetadata {
                        scopes,
                        reference,
                        kind: VariableKind::Wire,
                        width: signal.width() as u32,
                        index: None,
                    },
                    name,
                    signal,
                }
            })
            .collect::<Vec<_>>();
        Self::from_signals(signals, end_time)
    }

    pub fn from_vcd(signals: Vec<(vcd::ScopedVar, vcd::Signal)>, end_time: u64) -> Self {
        let signals = signals
            .into_iter()
            .enumerate()
            .map(|(signal_index, (scoped, signal))| {
                let scopes = scoped
                    .scopes
                    .into_iter()
                    .map(|(kind, name)| ScopeMetadata {
                        name,
                        kind: scope_kind(kind),
                    })
                    .collect::<Vec<_>>();
                let metadata = SignalMetadata {
                    scopes,
                    reference: scoped.var.reference,
                    kind: variable_kind(scoped.var.var_type),
                    width: scoped.var.size,
                    index: scoped.var.index.map(signal_index_from_vcd),
                };
                let name = full_name(&metadata);
                WaveformSignal {
                    id: SignalId::new(signal_index as u64),
                    name,
                    metadata,
                    signal,
                }
            })
            .collect();
        Self::from_signals(signals, end_time)
    }

    fn from_signals(signals: Vec<WaveformSignal>, end_time: u64) -> Self {
        let hierarchy = build_hierarchy(&signals);
        Self {
            signals,
            hierarchy,
            end_time,
        }
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

    pub fn hierarchy(&self) -> &SignalHierarchy {
        &self.hierarchy
    }

    pub fn end_time(&self) -> u64 {
        self.end_time
    }
}

fn full_name(metadata: &SignalMetadata) -> String {
    let mut name = metadata
        .scopes
        .iter()
        .map(|scope| scope.name.as_str())
        .collect::<Vec<_>>()
        .join(".");
    if !name.is_empty() {
        name.push('.');
    }
    name.push_str(&metadata.reference);
    name
}

fn build_hierarchy(signals: &[WaveformSignal]) -> SignalHierarchy {
    let mut hierarchy = SignalHierarchy::default();
    for signal in signals {
        let mut scopes = &mut hierarchy.scopes;
        let mut leaf_signals = &mut hierarchy.signals;
        for metadata in &signal.metadata.scopes {
            let index = scopes
                .iter()
                .position(|scope| scope.metadata == *metadata)
                .unwrap_or_else(|| {
                    scopes.push(ScopeNode {
                        metadata: metadata.clone(),
                        scopes: Vec::new(),
                        signals: Vec::new(),
                    });
                    scopes.len() - 1
                });
            let scope = &mut scopes[index];
            leaf_signals = &mut scope.signals;
            scopes = &mut scope.scopes;
        }
        leaf_signals.push(signal.id);
    }
    hierarchy
}

fn scope_kind(kind: ::vcd::ScopeType) -> ScopeKind {
    match kind {
        ::vcd::ScopeType::Module => ScopeKind::Module,
        ::vcd::ScopeType::Task => ScopeKind::Task,
        ::vcd::ScopeType::Function => ScopeKind::Function,
        ::vcd::ScopeType::Begin => ScopeKind::Begin,
        ::vcd::ScopeType::Fork => ScopeKind::Fork,
        _ => ScopeKind::Module,
    }
}

fn variable_kind(kind: ::vcd::VarType) -> VariableKind {
    match kind {
        ::vcd::VarType::Reg => VariableKind::Register,
        _ => VariableKind::Wire,
    }
}

fn signal_index_from_vcd(index: ::vcd::ReferenceIndex) -> SignalIndex {
    match index {
        ::vcd::ReferenceIndex::BitSelect(bit) => SignalIndex::Bit(bit),
        ::vcd::ReferenceIndex::Range(msb, lsb) => SignalIndex::Range(msb, lsb),
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
        assert_eq!(waveform.hierarchy().scopes().len(), 1);
        let top = &waveform.hierarchy().scopes()[0];
        assert_eq!(top.metadata().name(), "top");
        assert_eq!(top.signals(), &[SignalId::new(0), SignalId::new(1)]);
        assert_eq!(top.signal_ids_recursive(), top.signals());
    }

    #[test]
    fn builds_nested_scopes_without_flattening_signal_metadata() {
        let waveform = Waveform::new(
            vec![
                ("soc.cpu.clock".to_owned(), vcd::Signal::new(1)),
                ("soc.cpu.data".to_owned(), vcd::Signal::new(8)),
                ("soc.uart.tx".to_owned(), vcd::Signal::new(1)),
            ],
            10,
        );

        let soc = &waveform.hierarchy().scopes()[0];
        assert_eq!(soc.metadata().name(), "soc");
        assert_eq!(soc.scopes().len(), 2);
        assert_eq!(soc.signal_ids_recursive().len(), 3);
        let data = waveform.signal(SignalId::new(1)).unwrap();
        assert_eq!(data.metadata().reference(), "data");
        assert_eq!(data.metadata().width(), 8);
        assert_eq!(
            data.metadata()
                .scopes()
                .iter()
                .map(ScopeMetadata::name)
                .collect::<Vec<_>>(),
            vec!["soc", "cpu"]
        );
    }
}
