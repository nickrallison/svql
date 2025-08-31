use std::hash::Hash;
use std::{collections::HashMap, fmt::Formatter};

use prjunnamed_netlist::{Cell, CellRef, ControlNet, Design, Net, Trit, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NodeType {
    Buf,
    Not,
    And,
    Or,
    Xor,
    Mux,
    Adc,
    Aig,
    Eq,
    ULt,
    SLt,
    Shl,
    UShr,
    SShr,
    XShr,
    Mul,
    UDiv,
    UMod,
    SDivTrunc,
    SDivFloor,
    SModTrunc,
    SModFloor,
    Match,
    Assign,
    Dff,
    Memory,
    IoBuf,
    Target,
    Other,
    Input,
    Output,
    Name,
    Debug,
}

impl NodeType {
    pub fn is_logic_gate(&self) -> bool {
        matches!(
            self,
            NodeType::Buf
                | NodeType::Not
                | NodeType::And
                | NodeType::Or
                | NodeType::Xor
                | NodeType::Mux
                | NodeType::Adc
                | NodeType::Aig
                | NodeType::Eq
                | NodeType::ULt
                | NodeType::SLt
                | NodeType::Shl
                | NodeType::UShr
                | NodeType::SShr
                | NodeType::XShr
                | NodeType::Mul
                | NodeType::UDiv
                | NodeType::UMod
                | NodeType::SDivTrunc
                | NodeType::SDivFloor
                | NodeType::SModTrunc
                | NodeType::SModFloor
                | NodeType::Dff
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(self, NodeType::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, NodeType::Output)
    }

    pub fn has_commutative_inputs(&self) -> bool {
        matches!(
            self,
            NodeType::And | NodeType::Or | NodeType::Xor | NodeType::Aig
        )
    }
}

impl From<&Cell> for NodeType {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Buf(..) => NodeType::Buf,
            Cell::Not(..) => NodeType::Not,
            Cell::And(..) => NodeType::And,
            Cell::Or(..) => NodeType::Or,
            Cell::Xor(..) => NodeType::Xor,
            Cell::Mux(..) => NodeType::Mux,
            Cell::Adc(..) => NodeType::Adc,
            Cell::Aig(..) => NodeType::Aig,
            Cell::Eq(..) => NodeType::Eq,
            Cell::ULt(..) => NodeType::ULt,
            Cell::SLt(..) => NodeType::SLt,
            Cell::Shl(..) => NodeType::Shl,
            Cell::UShr(..) => NodeType::UShr,
            Cell::SShr(..) => NodeType::SShr,
            Cell::XShr(..) => NodeType::XShr,
            Cell::Mul(..) => NodeType::Mul,
            Cell::UDiv(..) => NodeType::UDiv,
            Cell::UMod(..) => NodeType::UMod,
            Cell::SDivTrunc(..) => NodeType::SDivTrunc,
            Cell::SDivFloor(..) => NodeType::SDivFloor,
            Cell::SModTrunc(..) => NodeType::SModTrunc,
            Cell::SModFloor(..) => NodeType::SModFloor,
            Cell::Match(..) => NodeType::Match,
            Cell::Assign(..) => NodeType::Assign,
            Cell::Dff(..) => NodeType::Dff,
            Cell::Memory(..) => NodeType::Memory,
            Cell::IoBuf(..) => NodeType::IoBuf,
            Cell::Target(..) => NodeType::Target,
            Cell::Other(..) => NodeType::Other,
            Cell::Input(..) => NodeType::Input,
            Cell::Output(..) => NodeType::Output,
            Cell::Name(..) => NodeType::Name,
            Cell::Debug(..) => NodeType::Debug,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Named fan-in representation using NodeSource to preserve Const/Gate/Io information
#[derive(Clone, Debug)]
pub struct NodeFanin<'a> {
    pub map: HashMap<String, Vec<NodeSource<'a>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeSource<'a> {
    Gate(CellRef<'a>, usize),
    Io(CellRef<'a>, usize),
    Const(Trit),
}

pub fn net_to_source<'a>(design: &'a Design, net: Net) -> NodeSource<'a> {
    match design.find_cell(net) {
        Ok((src, bit)) => {
            if NodeType::from(src.get().as_ref()).is_logic_gate() {
                NodeSource::Gate(src, bit)
            } else {
                NodeSource::Io(src, bit)
            }
        }
        Err(trit) => NodeSource::Const(trit),
    }
}

// Build a named‑port fan‑in map for a given cell.
// Port names are chosen to be stable and semantic. Both pattern and design
// nodes run through the same logic, so names align for matching.
pub fn fanin_named<'a>(design: &'a Design, cell: &Cell) -> NodeFanin<'a> {
    let mut map: HashMap<String, Vec<NodeSource<'a>>> = HashMap::new();

    let push_value = |mut map: HashMap<String, Vec<NodeSource<'a>>>, name: &str, v: &Value| {
        let e = map.entry(name.to_string()).or_default();
        for i in 0..v.len() {
            e.push(net_to_source(design, v[i]));
        }
        return map;
    };

    let push_net = |mut map: HashMap<String, Vec<NodeSource<'a>>>, name: &str, n: Net| {
        map.entry(name.to_string())
            .or_default()
            .push(net_to_source(design, n));
        return map;
    };

    let push_ctrl = |mut map: HashMap<String, Vec<NodeSource<'a>>>, name: &str, c: &ControlNet| {
        // Extract underlying net (ignoring polarity for connectivity match)
        let mut found: Option<Net> = None;
        c.visit(|n| {
            found = Some(n);
        });
        if let Some(n) = found {
            map = push_net(map, name, n);
        }
        return map;
    };

    match cell {
        Cell::Buf(v) => {
            map = push_value(map, "in", v);
        }
        Cell::Not(v) => {
            map = push_value(map, "in", v);
        }
        Cell::And(a, b) | Cell::Or(a, b) | Cell::Xor(a, b) => {
            map = push_value(map, "a", a);
            map = push_value(map, "b", b);
        }
        Cell::Mux(sel, t, f) => {
            map = push_net(map, "sel", *sel);
            map = push_value(map, "t", t);
            map = push_value(map, "f", f);
        }
        Cell::Adc(a, b, ci) => {
            map = push_value(map, "a", a);
            map = push_value(map, "b", b);
            map = push_net(map, "ci", *ci);
        }
        Cell::Aig(a, b) => {
            map = push_ctrl(map, "a", a);
            map = push_ctrl(map, "b", b);
        }
        Cell::Eq(a, b) | Cell::ULt(a, b) | Cell::SLt(a, b) => {
            map = push_value(map, "a", a);
            map = push_value(map, "b", b);
        }
        Cell::Shl(a, b, _) | Cell::UShr(a, b, _) | Cell::SShr(a, b, _) | Cell::XShr(a, b, _) => {
            map = push_value(map, "a", a);
            map = push_value(map, "b", b);
        }
        Cell::Mul(a, b)
        | Cell::UDiv(a, b)
        | Cell::UMod(a, b)
        | Cell::SDivTrunc(a, b)
        | Cell::SDivFloor(a, b)
        | Cell::SModTrunc(a, b)
        | Cell::SModFloor(a, b) => {
            map = push_value(map, "a", a);
            map = push_value(map, "b", b);
        }

        // Stateful / complex cells: name a reasonable subset for our matching.
        Cell::Dff(ff) => {
            map = push_value(map, "d", &ff.data);
            map = push_ctrl(map, "clk", &ff.clock);
            map = push_ctrl(map, "en", &ff.enable);
            map = push_ctrl(map, "reset", &ff.reset);
            map = push_ctrl(map, "clear", &ff.clear);
        }

        // Less common in our current patterns; add empty or minimal mapping.
        Cell::Memory(_) => todo!("Memory fan-in naming"),
        Cell::IoBuf(io) => {
            // io_buffer::IoBuffer expected fields: (io, output, dir)? Use reasonable names if present.
            map = push_value(map, "output", &io.output);
            map = push_ctrl(map, "enable", &io.enable);
        }
        Cell::Target(_) => {}
        Cell::Other(_) => {}

        // IO and meta: no fan-in.
        Cell::Input(..) => {}
        Cell::Output(_, v) => {
            map = push_value(map, "in", v);
        }
        Cell::Name(_, v) | Cell::Debug(_, v) => {
            map = push_value(map, "in", v);
        }

        // Match/Assign are uncommon; fall back to undifferentiated inputs
        Cell::Match(m) => {
            map = push_value(map, "value", &m.value);
        }
        Cell::Assign(a) => {
            map = push_value(map, "value", &a.value);
        }
    }

    NodeFanin { map }
}
