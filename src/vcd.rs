use indexmap::map::IndexMap;
use std::io;
// use std::slice::Chunks;

use std::collections::{btree_map, BTreeMap};
pub use vcd::Value;

#[derive(Debug)]
pub struct Signal {
    // index into the values
    ix: BTreeMap<u64, usize>,
    values: SignalValues,
    width: usize,
}

#[derive(Debug)]
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
        eprintln!("insert(time = {time}, value = {value:?})");
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
    let header = parser.parse_header()?;
    let mut id_map: IndexMap<vcd::IdCode, ScopedVar> = IndexMap::new();
    let mut signal_map: IndexMap<vcd::IdCode, Signal> = IndexMap::new();

    for item in header_vars(&header.items) {
        match item.var.var_type {
            vcd::VarType::Reg | vcd::VarType::Wire => (),
            _ => continue,
        }
        signal_map.insert(item.var.code, Signal::new(item.var.size as usize));
        id_map.insert(item.var.code, item);
    }

    let mut time = 0;

    for command in parser {
        use vcd::Command::*;
        // eprintln!("{command:?}");
        match command? {
            Timestamp(t) => time = t,
            ChangeScalar(i, v) => {
                let signal = signal_map.get_mut(&i).unwrap();
                signal.insert_bit(time, v);
            }
            ChangeVector(i, v) => {
                // panic!("can't change vector yet");
                if let Some(signal) = signal_map.get_mut(&i) {
                    signal.insert(time, v.into());
                } else {
                    eprintln!("id {i:?} not found");
                }
            }
            ChangeString(_i, s) => {
                eprintln!("I saw a ChangeString '{s}'");
            }
            _ => (),
        }
    }

    let mut vec_output = vec![];
    for (id, var) in id_map {
        let mut signal = signal_map.remove(&id).unwrap();
        if let Some((_, &last_v)) = signal.ix.iter().next_back() {
            signal.ix.insert(time, last_v);
        }
        vec_output.push((var, signal));
    }

    Ok((vec_output, time))
}
