use indexmap::map::IndexMap;
use std::io;
// use std::slice::Chunks;

use std::hash::{Hash, Hasher};

use std::collections::{btree_map, BTreeMap};
// pub use vcd::Value;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Signal {
    // index into the values
    ix: BTreeMap<u64, usize>,
    values: SignalValues,
    width: usize,
}

impl Hash for Signal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ix.hash(state);
        // vcd::Value is not hashable, so we have to wrap it
        // let values: Vec<V> = match &self.values {
        //     SignalValues::Values(vs) => vs.iter().map(|v| V(*v)).collect(),
        // };
        // values.hash(state);
        self.width.hash(state);
    }
}

impl From<vcd::Value> for Value {
    fn from(v: vcd::Value) -> Self {
        match v {
            vcd::Value::V0 => Value::V0,
            vcd::Value::V1 => Value::V1,
            vcd::Value::X => Value::X,
            vcd::Value::Z => Value::Z,
        }
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, serde::Serialize, serde::Deserialize,
)]
pub enum Value {
    /// Logic low
    ///
    /// (prefixed with `V` to make a valid Rust identifier)
    V0,

    /// Logic high
    ///
    /// (prefixed with `V` to make a valid Rust identifier)
    V1,

    /// An uninitialized or unknown value
    X,

    /// The "high-impedance" value
    Z,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
enum SignalValues {
    // done in chunks of the signal width
    Values(Vec<Value>),
    // Floats(Vec<f64>),
    // could be single vector of bytes with null terminated strings to reduce allocations
    // Strings(Vec<String>),
}

impl Signal {
    pub fn new(width: usize) -> Signal {
        Signal {
            ix: BTreeMap::new(),
            values: SignalValues::Values(vec![]),
            width,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn is_empty(&self) -> bool {
        self.ix.is_empty()
    }

    pub fn final_time(&self) -> u64 {
        self.ix
            .iter()
            .next_back()
            .as_ref()
            .map(|x| *x.0)
            .unwrap_or(0)
    }

    pub fn previous_transition(&self, time: u64) -> Option<u64> {
        self.ix.range(..time).next_back().map(|(&time, _)| time)
    }

    pub fn next_transition(&self, time: u64) -> Option<u64> {
        self.ix
            .range((std::ops::Bound::Excluded(time), std::ops::Bound::Unbounded))
            .next()
            .map(|(&time, _)| time)
    }

    pub fn value_at(&self, time: u64) -> Option<&[Value]> {
        let (_, &start) = self.ix.range(..=time).next_back()?;
        match &self.values {
            SignalValues::Values(values) => values.get(start..start + self.width),
        }
    }

    // pub fn scalars(&self) -> impl Iterator<Item = (u64, Value)> + '_ {
    //     // assert!(self.width == 1);
    //     self.values.iter().map(|(&k, ix)| (k, v[0]))
    // }

    // pub fn range(&self, range: std::ops::Range<u64>) -> btree_map::Range<'_, u64, Vec<Value>> {
    //     let lower_bound = *self.values.range(..range.start).next_back().as_ref().unwrap().0;
    //     self.values.range(lower_bound..range.end)
    // }

    pub fn bit_range(&self, range: std::ops::Range<u64>) -> BitSignalRange<'_> {
        BitSignalRange {
            map: &self.ix,
            range,
            values: match &self.values {
                SignalValues::Values(vs) => vs,
                // _ => panic!("bit_range"),
            },
        }
    }

    pub fn range(&self, range: std::ops::Range<u64>) -> SignalRange<'_> {
        SignalRange {
            map: &self.ix,
            range,
            width: self.width,
            values: match &self.values {
                SignalValues::Values(vs) => vs,
            },
        }
    }

    pub fn insert_bit(&mut self, time: u64, value: Value) {
        if self.width != 1 {
            panic!("insert bit: width {} != 1", self.width);
        }
        // eprintln!("inserting bit at time {time}");
        match &mut self.values {
            SignalValues::Values(vs) => {
                vs.push(value);
                let ix = vs.len() - 1;
                self.ix.insert(time, ix);
            } // _ => panic!("insert_bit into non-value"),
        }
    }

    pub fn insert(&mut self, time: u64, value: Vec<Value>) {
        // eprintln!("insert(time = {time}, value = {value:?})");
        // if self.width != value.len() {
        //     eprintln!("{} != {}", self.width, value.len());
        // }
        assert!(value.len() <= self.width);
        match &mut self.values {
            SignalValues::Values(vs) => {
                let ix = vs.len();
                vs.extend(value.iter().cloned());
                for _ in 0..(self.width - value.len()) {
                    vs.push(Value::V0);
                }
                self.ix.insert(time, ix);
            }
        }
    }
}

impl std::ops::Index<u64> for Signal {
    type Output = [Value];
    fn index(&self, index: u64) -> &[Value] {
        self.range(0..index)
            .into_iter()
            .next_back()
            .as_ref()
            .unwrap()
            .1
    }
}

pub struct SignalRange<'a> {
    map: &'a BTreeMap<u64, usize>,
    range: std::ops::Range<u64>,
    width: usize,
    values: &'a [Value],
}

pub struct SignalRangeIter<'a> {
    iter: btree_map::Range<'a, u64, usize>,
    width: usize,
    values: &'a [Value],
}

impl<'a> Iterator for SignalRangeIter<'a> {
    type Item = (u64, &'a [Value]);
    fn next(&mut self) -> Option<Self::Item> {
        let (t, ix) = self.iter.next()?;
        // eprintln!("let (t = {t}, ix = {ix}) = self.iter.next()?;");
        // let max_ix = (self.values.len() / self.width) - 1; // XXX THIS IS WRONG, WHAT'S GOING ON HERE?
        let start = *ix;
        let end = start + self.width;
        Some((*t, &self.values[start..end]))
    }
}

impl<'a> DoubleEndedIterator for SignalRangeIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (t, ix) = self.iter.next_back()?;
        let start = *ix;
        let end = start + self.width;
        Some((*t, &self.values[start..end]))
    }
}

// impl<'a> Index<u64> for SignalRange<'a> {
//     type Output = &'a [Value];
//     fn index(&self, index: u64) -> &[Value] {
//         self.map.range(..index).next_back().as_ref().unwrap().1
//     }
// }

impl<'a> IntoIterator for SignalRange<'a> {
    type Item = (u64, &'a [Value]);
    type IntoIter = SignalRangeIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let lower_bound = self
            .map
            .range(..self.range.start)
            .next_back()
            .as_ref()
            .map_or(0, |v| *v.0);
        let upper_bound = self
            .map
            .range(self.range.end..)
            .next()
            .as_ref()
            .map_or(u64::MAX - 1, |v| *v.0);
        let iter = self.map.range(lower_bound..upper_bound + 1);
        SignalRangeIter {
            iter,
            width: self.width,
            values: self.values,
        }
    }
}

pub struct BitSignalRange<'a> {
    map: &'a BTreeMap<u64, usize>,
    range: std::ops::Range<u64>,
    values: &'a [Value],
}

pub struct BitSignalRangeIter<'a> {
    iter: btree_map::Range<'a, u64, usize>,
    values: &'a [Value],
}

impl<'a> Iterator for BitSignalRangeIter<'a> {
    type Item = (u64, Value);
    fn next(&mut self) -> Option<Self::Item> {
        let (t, ix) = self.iter.next()?;
        Some((*t, self.values[*ix]))
    }
}

impl<'a> IntoIterator for BitSignalRange<'a> {
    type Item = (u64, Value);
    type IntoIter = BitSignalRangeIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        let lower_bound = self
            .map
            .range(..self.range.start)
            .next_back()
            .as_ref()
            .map_or(0, |v| *v.0);
        let upper_bound = self
            .map
            .range(self.range.end..)
            .next()
            .as_ref()
            .map_or(u64::MAX - 1, |v| *v.0);
        let iter = self.map.range(lower_bound..upper_bound + 1);
        BitSignalRangeIter {
            iter,
            values: self.values,
        }
    }
}

#[derive(Debug)]
pub struct ScopedVar {
    pub scopes: Vec<(vcd::ScopeType, String)>,
    pub var: vcd::Var,
}

fn header_vars(items: &[vcd::ScopeItem]) -> Vec<ScopedVar> {
    let mut vars = vec![];

    fn add_scopes(
        vars: &mut Vec<ScopedVar>,
        stack: &[(vcd::ScopeType, String)],
        scope_item: &vcd::ScopeItem,
    ) {
        match scope_item {
            vcd::ScopeItem::Var(var) => vars.push(ScopedVar {
                scopes: stack.to_vec(),
                var: var.clone(),
            }),
            vcd::ScopeItem::Scope(scope) => {
                let mut stack = stack.to_vec();
                stack.push((scope.scope_type, scope.identifier.clone()));
                scope
                    .items
                    .iter()
                    .for_each(|item| add_scopes(vars, &stack, item));
            }
            vcd::ScopeItem::Comment(_) => {}
            _ => (),
        }
    }

    items
        .iter()
        .for_each(|item| add_scopes(&mut vars, &[], item));

    // for var in &vars {
    //     eprintln!("var = {var:?}");
    // }

    vars
}

pub fn read_clocked_vcd(
    r: &mut impl io::BufRead,
) -> std::io::Result<(Vec<(ScopedVar, Signal)>, u64)> {
    let mut parser = vcd::Parser::new(r);

    // The VCD spec is weird and confusing. There's a couple of features I'm not bothering to
    // impliment yet (and probably others I've missed or misunderstood):
    //
    //   - the spec allows multiple variables to have the same identifier code which I don't support
    //     (shouldn't be too difficuilt to add)
    //   - the reference indexes are ignored, I don't really understand why you'd want it and it's
    //     annoying the resolve the types (but also shouldn't be that difficult)
    //
    // I assume that time isn't allowed to go backwards but I don't think this is explicit in the
    // spec. This implimentation allows going backwards in time only to change signals whose values
    // for a later time haven't yet been written. i.e. each individual signal needs monotonous times
    // but this can be interleaved in the vcd files.

    // Parse the header and find the wires
    let header = match parser.parse_header() {
        Ok(header) => header,
        Err(err) => {
            if let Some(err2) = err.get_ref() {
                if let Some(err) = err2.downcast_ref::<vcd::ParseError>() {
                    log::warn!("vcd header parse error: {}", err);
                }
                return Err(err);
            } else {
                return Err(err);
            }
        }
    };
    let mut id_map: IndexMap<vcd::IdCode, ScopedVar> = IndexMap::new();
    let mut signal_map: IndexMap<vcd::IdCode, Signal> = IndexMap::new();

    for item in header_vars(&header.items) {
        match item.var.var_type {
            // Real and String variables carry ChangeReal/ChangeString rather than
            // scalar/vector bit changes, so they aren't representable as bit-vector
            // signals here. Every other type (Wire, Reg, Integer, Time, Tri, WAnd, …)
            // is a bit vector and is registered so its value changes have a home.
            vcd::VarType::Real | vcd::VarType::String => continue,
            _ => (),
        }
        signal_map.insert(item.var.code, Signal::new(item.var.size as usize));
        id_map.insert(item.var.code, item);
    }

    let mut time = 0;

    while let Some(command) = parser.next() {
        use vcd::Command::*;
        match command {
            Ok(Timestamp(t)) => time = t,
            Ok(ChangeScalar(i, v)) => match signal_map.get_mut(&i) {
                Some(signal) => signal.insert(time, vec![v.into()]),
                None => log::warn!("ChangeScalar id {i:?} not found"),
            },
            Ok(ChangeVector(i, v)) => {
                // panic!("can't change vector yet");
                if let Some(signal) = signal_map.get_mut(&i) {
                    signal.insert(time, v.iter().map(|x| x.into()).collect());
                } else {
                    log::warn!("id {i:?} not found");
                }
            }
            Err(err) => {
                if let Some(err) = err.get_ref() {
                    if let Some(err) = err.downcast_ref::<vcd::ParseError>() {
                        log::warn!("vcd parse error: {}", err);
                        parser.reader().read_line(&mut String::new())?;
                    }
                } else {
                    return Err(err);
                }
            }
            _ => (),
        }
    }

    let mut vec_output = vec![];
    for (id, var) in id_map {
        let mut signal = signal_map.swap_remove(&id).unwrap();
        if let Some((_, &last_v)) = signal.ix.iter().next_back() {
            signal.ix.insert(time, last_v);
        }
        vec_output.push((var, signal));
    }

    Ok((vec_output, time))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> (Vec<(ScopedVar, Signal)>, u64) {
        read_clocked_vcd(&mut std::io::Cursor::new(input.as_bytes())).unwrap()
    }

    const HEADER: &str = "\
$timescale 1 ns $end
$scope module top $end
$var wire 1 ! clock $end
$var wire 4 \" count [3:0] $end
$upscope $end
$enddefinitions $end
";

    #[test]
    fn parses_scalar_and_vector_transitions() {
        let (signals, final_time) = parse(&format!(
            "{HEADER}#0\n0!\nb0000 \"\n#5\n1!\nb1010 \"\n#10\n0!\n"
        ));

        assert_eq!(final_time, 10);
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].0.scopes[0].1, "top");
        assert_eq!(signals[0].0.var.reference, "clock");
        assert_eq!(
            signals[0]
                .1
                .bit_range(0..11)
                .into_iter()
                .collect::<Vec<_>>(),
            vec![(0, Value::V0), (5, Value::V1), (10, Value::V0),]
        );
        let vector_at_five = signals[1]
            .1
            .range(5..6)
            .into_iter()
            .find(|(time, _)| *time == 5)
            .unwrap()
            .1;
        assert_eq!(
            vector_at_five,
            [Value::V1, Value::V0, Value::V1, Value::V0,]
        );
        assert_eq!(signals[1].1.value_at(4), Some(&[Value::V0; 4][..]));
        assert_eq!(signals[1].1.value_at(5), Some(vector_at_five));
        assert_eq!(signals[1].1.value_at(100), Some(vector_at_five));
    }

    #[test]
    fn ignores_non_wire_variables() {
        let input = "\
$timescale 1 ns $end
$scope module top $end
$var real 1 ! analog $end
$var wire 1 \" digital $end
$upscope $end
$enddefinitions $end
#0
r1.5 !
0\"
";
        let (signals, _) = parse(input);
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].0.var.reference, "digital");
    }

    #[test]
    fn waveform_preserves_scope_variable_and_index_metadata() {
        let input = "\
$timescale 1 ns $end
$scope module soc $end
$scope task worker $end
$var reg 8 ! accumulator [7:0] $end
$upscope $end
$upscope $end
$enddefinitions $end
#0
b00000000 !
";
        let (signals, end_time) = parse(input);
        let waveform = crate::waveform::Waveform::from_vcd(signals, end_time);
        let signal = &waveform.signals()[0];

        assert_eq!(signal.name(), "soc.worker.accumulator");
        assert_eq!(
            signal.metadata().kind(),
            crate::waveform::VariableKind::Register
        );
        assert_eq!(
            signal.metadata().index(),
            Some(crate::waveform::SignalIndex::Range(7, 0))
        );
        assert_eq!(
            waveform.hierarchy().scopes()[0].scopes()[0]
                .metadata()
                .kind(),
            crate::waveform::ScopeKind::Task
        );
    }

    #[test]
    fn reports_a_truncated_header() {
        assert!(read_clocked_vcd(&mut std::io::Cursor::new(b"$scope module top $end\n")).is_err());
    }

    #[test]
    fn parses_the_edge_case_fixture() {
        // Keep this input in sync with the user-loadable `vcds/edge_cases.vcd` fixture. The test
        // is deliberately self-contained because Git-backed Nix flakes omit new untracked files,
        // and the development plan is kept uncommitted while it is being executed.
        let input = r#"$timescale 1 ns $end
$scope module top $end
$var wire 1 ! scalar $end
$var wire 4 " bus [3:0] $end
$var wire 1 # repeated $end
$upscope $end
$enddefinitions $end
#0
x!
bxxxx "
0#
#5
z!
bz01x "
0#
#10
1!
b1010 "
#15
"#;
        let (signals, final_time) = parse(input);

        assert_eq!(final_time, 15);
        assert_eq!(signals.len(), 3);

        let signal = |name: &str| {
            &signals
                .iter()
                .find(|(var, _)| var.var.reference == name)
                .unwrap()
                .1
        };

        assert_eq!(
            signal("scalar")
                .bit_range(0..16)
                .into_iter()
                .collect::<Vec<_>>(),
            vec![
                (0, Value::X),
                (5, Value::Z),
                (10, Value::V1),
                (15, Value::V1),
            ]
        );
        assert_eq!(
            signal("repeated")
                .bit_range(0..16)
                .into_iter()
                .collect::<Vec<_>>(),
            vec![(0, Value::V0), (5, Value::V0), (15, Value::V0)]
        );
        assert_eq!(
            signal("bus")
                .range(0..16)
                .into_iter()
                .map(|(time, values)| (time, values.to_vec()))
                .collect::<Vec<_>>(),
            vec![
                (0, vec![Value::X, Value::X, Value::X, Value::X]),
                (5, vec![Value::Z, Value::V0, Value::V1, Value::X]),
                (10, vec![Value::V1, Value::V0, Value::V1, Value::V0]),
                (15, vec![Value::V1, Value::V0, Value::V1, Value::V0]),
            ]
        );
    }
}
