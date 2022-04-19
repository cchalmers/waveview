use std::io::ErrorKind::InvalidInput;
use indexmap::map::IndexMap;
use std::io;
// use std::slice::Chunks;

pub use vcd::Value;
use std::collections::{btree_map, BTreeMap};

use std::ops::Index;

#[derive(Debug)]
pub struct Signal {
    values: BTreeMap<u64, Vec<Value>>,
    width: usize,
}

impl Signal {
    pub fn new(width: usize) -> Signal {
        Signal {
            values: BTreeMap::new(),
            width,
        }
    }

    pub fn scalars(&self) -> impl Iterator<Item = (u64, Value)> + '_ {
        // assert!(self.width == 1);
        self.values.iter().map(|(&k, v)| (k, v[0]))
    }

    pub fn range(&self, range: std::ops::Range<u64>) -> btree_map::Range<'_, u64, Vec<Value>> {
        let lower_bound = *self.values.range(..range.start).next_back().as_ref().unwrap().0;
        self.values.range(lower_bound..range.end)
    }

    pub fn insert(&mut self, time: u64, value: Vec<Value>) {
        if self.width != value.len() {
            eprintln!("{} != {}", self.width, value.len());
        }
        self.values.insert(time, value);
    }
}

impl Index<u64> for Signal {
    type Output = [Value];
    fn index(&self, index: u64) -> &[Value] {
        self.values.range(..index).next_back().as_ref().unwrap().1
    }
}

struct SignalSlice<'a> {
    map: &'a BTreeMap<u64, Vec<Value>>,
    range: std::ops::Range<u64>,
    width: usize,
}

impl<'a> Index<u64> for SignalSlice<'a> {
    type Output = [Value];
    fn index(&self, index: u64) -> &[Value] {
        self.map.range(..index).next_back().as_ref().unwrap().1
    }
}


impl<'a> IntoIterator for SignalSlice<'a> {
    type Item = (&'a u64, &'a Vec<Value>);
    type IntoIter = btree_map::Range<'a, u64, Vec<Value>>;
    fn into_iter(self) -> Self::IntoIter {
        let lower_bound = *self.map.range(..self.range.start).next_back().as_ref().unwrap().0;
        self.map.range(lower_bound..self.range.end)
    }
}

#[derive(Debug)]
pub struct ScopedVar {
    pub scopes: Vec<(vcd::ScopeType, String)>,
    pub var: vcd::Var,
}

fn header_vars(items: &[vcd::ScopeItem]) -> Vec<ScopedVar> {
    let mut vars = vec![];

    fn add_scopes(vars: &mut Vec<ScopedVar>, stack: &[(vcd::ScopeType, String)], scope_item: &vcd::ScopeItem) {
        match scope_item {
            vcd::ScopeItem::Var(var) => vars.push(ScopedVar { scopes: stack.to_vec(), var: var.clone() }),
            vcd::ScopeItem::Scope(scope) => {
                let mut stack = stack.to_vec();
                stack.push((scope.scope_type, scope.identifier.clone()));
                scope.children.iter().for_each(|item| add_scopes(vars, &stack, item));
            }
        }
    }

    items.iter().for_each(|item| add_scopes(&mut vars, &[], item));

    for var in &vars {
        eprintln!("var = {var:?}");
    }

    vars
}

pub fn read_clocked_vcd(r: &mut impl io::Read) -> std::io::Result<Vec<(ScopedVar, Signal)>>  {
   let mut parser = vcd::Parser::new(r);

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
     eprintln!("{command:?}");
     match command? {
       Timestamp(t) => time = t,
       ChangeScalar(i, v) => {
         let signal = signal_map.get_mut(&i).unwrap();
         signal.insert(time, vec![v]);
       }
       ChangeVector(i, v) => {
         if let Some(signal) = signal_map.get_mut(&i) {
            signal.insert(time, v);
         } else {
             eprintln!("id {i:?} not found");
         }
       }
       _ => (),
     }
   }

   let mut vec_output = vec![];
   for (id, var) in id_map {
       let signal = signal_map.remove(&id).unwrap();
       vec_output.push((var, signal));
   }

   Ok(vec_output)
}
