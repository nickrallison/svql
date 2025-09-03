use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;
use std::{collections::HashMap, fmt::Formatter};

use prjunnamed_netlist::{Cell, CellRef, ControlNet, Design, Net, Trit, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CellType {
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

impl CellType {
    pub fn is_logic_gate(&self) -> bool {
        matches!(
            self,
            CellType::Buf
                | CellType::Not
                | CellType::And
                | CellType::Or
                | CellType::Xor
                | CellType::Mux
                | CellType::Adc
                | CellType::Aig
                | CellType::Eq
                | CellType::ULt
                | CellType::SLt
                | CellType::Shl
                | CellType::UShr
                | CellType::SShr
                | CellType::XShr
                | CellType::Mul
                | CellType::UDiv
                | CellType::UMod
                | CellType::SDivTrunc
                | CellType::SDivFloor
                | CellType::SModTrunc
                | CellType::SModFloor
                | CellType::Dff
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(self, CellType::Input)
    }

    pub fn is_output(&self) -> bool {
        matches!(self, CellType::Output)
    }

    pub fn has_commutative_inputs(&self) -> bool {
        matches!(
            self,
            CellType::And | CellType::Or | CellType::Xor | CellType::Aig
        )
    }
}

impl From<&Cell> for CellType {
    fn from(cell: &Cell) -> Self {
        match cell {
            Cell::Buf(..) => CellType::Buf,
            Cell::Not(..) => CellType::Not,
            Cell::And(..) => CellType::And,
            Cell::Or(..) => CellType::Or,
            Cell::Xor(..) => CellType::Xor,
            Cell::Mux(..) => CellType::Mux,
            Cell::Adc(..) => CellType::Adc,
            Cell::Aig(..) => CellType::Aig,
            Cell::Eq(..) => CellType::Eq,
            Cell::ULt(..) => CellType::ULt,
            Cell::SLt(..) => CellType::SLt,
            Cell::Shl(..) => CellType::Shl,
            Cell::UShr(..) => CellType::UShr,
            Cell::SShr(..) => CellType::SShr,
            Cell::XShr(..) => CellType::XShr,
            Cell::Mul(..) => CellType::Mul,
            Cell::UDiv(..) => CellType::UDiv,
            Cell::UMod(..) => CellType::UMod,
            Cell::SDivTrunc(..) => CellType::SDivTrunc,
            Cell::SDivFloor(..) => CellType::SDivFloor,
            Cell::SModTrunc(..) => CellType::SModTrunc,
            Cell::SModFloor(..) => CellType::SModFloor,
            Cell::Match(..) => CellType::Match,
            Cell::Assign(..) => CellType::Assign,
            Cell::Dff(..) => CellType::Dff,
            Cell::Memory(..) => CellType::Memory,
            Cell::IoBuf(..) => CellType::IoBuf,
            Cell::Target(..) => CellType::Target,
            Cell::Other(..) => CellType::Other,
            Cell::Input(..) => CellType::Input,
            Cell::Output(..) => CellType::Output,
            Cell::Name(..) => CellType::Name,
            Cell::Debug(..) => CellType::Debug,
        }
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CellWrapper<'a> {
    cell: Cow<'a, Cell>,
    cell_ref: CellRef<'a>,
}

impl<'a> CellWrapper<'a> {
    pub fn cell_type(&self) -> CellType {
        CellType::from(self.cell.as_ref())
    }
    pub fn get(&'a self) -> &'a Cell {
        self.cell.as_ref()
    }
    pub fn cell_ref(&'a self) -> CellRef<'a> {
        self.cell_ref
    }
    pub fn debug_index(&self) -> usize {
        self.cell_ref.debug_index()
    }

    pub fn input_name(&self) -> Option<&'a str> {
        match self.cell_ref.get() {
            std::borrow::Cow::Borrowed(Cell::Input(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn output_name(&self) -> Option<&'a str> {
        match self.cell_ref.get() {
            std::borrow::Cow::Borrowed(Cell::Output(name, _)) => Some(name.as_str()),
            _ => None,
        }
    }
}

impl<'a> Into<CellWrapper<'a>> for CellRef<'a> {
    fn into(self) -> CellWrapper<'a> {
        CellWrapper {
            cell: self.get(),
            cell_ref: self,
        }
    }
}

// // Named fan-in representation using NodeSource to preserve Const/Gate/Io information
// #[derive(Clone, Debug)]
// pub struct NodeFanin<'a> {
//     pub map: HashMap<String, Vec<NodeSource<'a>>>,
// }

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub enum NodeSource<'a> {
//     Gate(CellRef<'a>, usize),
//     Io(CellRef<'a>, usize),
//     Const(Trit),
// }

// pub fn net_to_source<'a>(design: &'a Design, net: Net) -> NodeSource<'a> {
//     match design.find_cell(net) {
//         Ok((src, bit)) => {
//             if CellType::from(src.get().as_ref()).is_logic_gate() {
//                 NodeSource::Gate(src, bit)
//             } else {
//                 NodeSource::Io(src, bit)
//             }
//         }
//         Err(trit) => NodeSource::Const(trit),
//     }
// }

// // Build a named‑port fan‑in map for a given cell.
// // Port names are chosen to be stable and semantic. Both pattern and design
// // nodes run through the same logic, so names align for matching.
// pub fn fanin_named<'a>(design: &'a Design, cell: &Cell) -> NodeFanin<'a> {
//     let mut map: HashMap<String, Vec<NodeSource<'a>>> = HashMap::new();

//     let push_value = |mut map: HashMap<String, Vec<NodeSource<'a>>>, name: &str, v: &Value| {
//         let e = map.entry(name.to_string()).or_default();
//         for i in 0..v.len() {
//             e.push(net_to_source(design, v[i]));
//         }
//         return map;
//     };

//     let push_net = |mut map: HashMap<String, Vec<NodeSource<'a>>>, name: &str, n: Net| {
//         map.entry(name.to_string())
//             .or_default()
//             .push(net_to_source(design, n));
//         return map;
//     };

//     let push_ctrl = |mut map: HashMap<String, Vec<NodeSource<'a>>>, name: &str, c: &ControlNet| {
//         // Extract underlying net (ignoring polarity for connectivity match)
//         let mut found: Option<Net> = None;
//         c.visit(|n| {
//             found = Some(n);
//         });
//         if let Some(n) = found {
//             map = push_net(map, name, n);
//         }
//         return map;
//     };

//     match cell {
//         Cell::Buf(v) => {
//             map = push_value(map, "in", v);
//         }
//         Cell::Not(v) => {
//             map = push_value(map, "in", v);
//         }
//         Cell::And(a, b) | Cell::Or(a, b) | Cell::Xor(a, b) => {
//             map = push_value(map, "a", a);
//             map = push_value(map, "b", b);
//         }
//         Cell::Mux(sel, t, f) => {
//             map = push_net(map, "sel", *sel);
//             map = push_value(map, "t", t);
//             map = push_value(map, "f", f);
//         }
//         Cell::Adc(a, b, ci) => {
//             map = push_value(map, "a", a);
//             map = push_value(map, "b", b);
//             map = push_net(map, "ci", *ci);
//         }
//         Cell::Aig(a, b) => {
//             map = push_ctrl(map, "a", a);
//             map = push_ctrl(map, "b", b);
//         }
//         Cell::Eq(a, b) | Cell::ULt(a, b) | Cell::SLt(a, b) => {
//             map = push_value(map, "a", a);
//             map = push_value(map, "b", b);
//         }
//         Cell::Shl(a, b, _) | Cell::UShr(a, b, _) | Cell::SShr(a, b, _) | Cell::XShr(a, b, _) => {
//             map = push_value(map, "a", a);
//             map = push_value(map, "b", b);
//         }
//         Cell::Mul(a, b)
//         | Cell::UDiv(a, b)
//         | Cell::UMod(a, b)
//         | Cell::SDivTrunc(a, b)
//         | Cell::SDivFloor(a, b)
//         | Cell::SModTrunc(a, b)
//         | Cell::SModFloor(a, b) => {
//             map = push_value(map, "a", a);
//             map = push_value(map, "b", b);
//         }

//         // Stateful / complex cells: name a reasonable subset for our matching.
//         Cell::Dff(ff) => {
//             map = push_value(map, "d", &ff.data);
//             map = push_ctrl(map, "clk", &ff.clock);
//             map = push_ctrl(map, "en", &ff.enable);
//             map = push_ctrl(map, "reset", &ff.reset);
//             map = push_ctrl(map, "clear", &ff.clear);
//         }

//         // Less common in our current patterns; add empty or minimal mapping.
//         Cell::Memory(_) => todo!("Memory fan-in naming"),
//         Cell::IoBuf(io) => {
//             // io_buffer::IoBuffer expected fields: (io, output, dir)? Use reasonable names if present.
//             map = push_value(map, "output", &io.output);
//             map = push_ctrl(map, "enable", &io.enable);
//         }
//         Cell::Target(_) => {}
//         Cell::Other(_) => {}

//         // IO and meta: no fan-in.
//         Cell::Input(..) => {}
//         Cell::Output(_, v) => {
//             map = push_value(map, "in", v);
//         }
//         Cell::Name(_, v) | Cell::Debug(_, v) => {
//             map = push_value(map, "in", v);
//         }

//         // Match/Assign are uncommon; fall back to undifferentiated inputs
//         Cell::Match(m) => {
//             map = push_value(map, "value", &m.value);
//         }
//         Cell::Assign(a) => {
//             map = push_value(map, "value", &a.value);
//         }
//     }

//     NodeFanin { map }
// }
