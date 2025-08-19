use std::collections::HashMap;

use prjunnamed_netlist::{Cell, Design, Net, Value};

use svql_subgraph::SubgraphMatch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NetId(Net);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PortBit(pub usize);

pub trait MatchBinding {
    fn output_driver(&self, port_name: &str, bit: PortBit) -> Option<NetId>;
    fn input_source(&self, port_name: &str, bit: PortBit) -> Option<NetId>;

    fn iter_inputs(&self) -> Box<dyn Iterator<Item = (String, PortBit, NetId)> + '_>;
    fn iter_outputs(&self) -> Box<dyn Iterator<Item = (String, PortBit, NetId)> + '_>;
}

#[derive(Clone, Debug, Default)]
pub struct SimpleMatchBinding {
    inputs: HashMap<(String, usize), NetId>,
    outputs: HashMap<(String, usize), NetId>,
}

impl MatchBinding for SimpleMatchBinding {
    fn output_driver(&self, port_name: &str, bit: PortBit) -> Option<NetId> {
        self.outputs.get(&(port_name.to_string(), bit.0)).copied()
    }

    fn input_source(&self, port_name: &str, bit: PortBit) -> Option<NetId> {
        self.inputs.get(&(port_name.to_string(), bit.0)).copied()
    }

    fn iter_inputs(&self) -> Box<dyn Iterator<Item = (String, PortBit, NetId)> + '_> {
        Box::new(
            self.inputs
                .iter()
                .map(|((name, bit), net)| (name.clone(), PortBit(*bit), *net)),
        )
    }

    fn iter_outputs(&self) -> Box<dyn Iterator<Item = (String, PortBit, NetId)> + '_> {
        Box::new(
            self.outputs
                .iter()
                .map(|((name, bit), net)| (name.clone(), PortBit(*bit), *net)),
        )
    }
}

pub fn binding_from_subgraph<'p, 'd>(m: &SubgraphMatch<'p, 'd>) -> SimpleMatchBinding {
    let mut inputs: HashMap<(String, usize), NetId> = HashMap::new();
    let mut outputs: HashMap<(String, usize), NetId> = HashMap::new();

    for ic in &m.pat_input_cells {
        if let Some(name) = ic.name() {
            // Pull bit width from the Cell::Input value
            let width = port_width_from_input(&ic.cref.cref().get());
            for bit in 0..width {
                if let Some((d_cell, d_bit)) = m.design_source_of_input_bit(name, bit) {
                    let net = d_cell.output()[d_bit];
                    inputs.insert((name.to_string(), bit), NetId(net));
                }
            }
        }
    }

    for oc in &m.pat_output_cells {
        if let Some(name) = oc.name() {
            let width = port_width_from_output(&oc.cref.cref().get());
            for bit in 0..width {
                if let Some((d_cell, d_bit)) = m.design_driver_of_output_bit(name, bit) {
                    let net = d_cell.output()[d_bit];
                    outputs.insert((name.to_string(), bit), NetId(net));
                }
            }
        }
    }

    SimpleMatchBinding { inputs, outputs }
}

fn port_width_from_input(cell: &std::borrow::Cow<'_, prjunnamed_netlist::Cell>) -> usize {
    match cell.as_ref() {
        Cell::Input(_, w) => *w,
        _ => 0,
    }
}

fn port_width_from_output(cell: &std::borrow::Cow<'_, prjunnamed_netlist::Cell>) -> usize {
    match cell.as_ref() {
        Cell::Output(_, v) => value_width(v),
        _ => 0,
    }
}

fn value_width(v: &Value) -> usize {
    v.iter().count()
}

pub fn find_bindings(
    pattern: &Design,
    design: &Design,
    config: &svql_subgraph::config::Config,
) -> Vec<SimpleMatchBinding> {
    let all = svql_subgraph::find_subgraphs(pattern, design, config);
    all.iter().map(binding_from_subgraph).collect()
}
