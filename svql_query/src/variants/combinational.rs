use crate::primitives::and::AndGate;
use crate::primitives::buf::BufGate;
use crate::primitives::mux2::Mux2Gate;
use crate::primitives::not::NotGate;
use crate::primitives::or::OrGate;
use crate::primitives::xnor::XnorGate;
use crate::primitives::xor::XorGate;
use crate::{State, Wire};
use svql_macros::variant;

#[variant(ports(y))]
pub enum Combinational<S: State> {
    // AND variants (multi-style implementations)
    #[variant(map(y = "y"))]
    AndGate(AndGate<S>),

    // OR gate
    #[variant(map(y = "y"))]
    Or(OrGate<S>),

    // XOR/XNOR gates
    #[variant(map(y = "y"))]
    Xor(XorGate<S>),
    #[variant(map(y = "y"))]
    Xnor(XnorGate<S>),

    // Single-input gates
    #[variant(map(y = "y"))]
    Not(NotGate<S>),
    #[variant(map(y = "y"))]
    Buf(BufGate<S>),

    // MUX (conditional combinational)
    #[variant(map(y = "y"))]
    Mux2(Mux2Gate<S>),
}

impl<S: State> Combinational<S> {
    /// Returns a descriptive name for the matched gate type.
    pub fn gate_type(&self) -> &'static str {
        match self {
            Combinational::AndGate(_) => "AND Gate",
            Combinational::Or(_) => "OR Gate",
            Combinational::Xor(_) => "XOR Gate",
            Combinational::Xnor(_) => "XNOR Gate",
            Combinational::Not(_) => "NOT Gate",
            Combinational::Buf(_) => "Buffer Gate",
            Combinational::Mux2(_) => "2:1 MUX Gate",
            _ => "Unknown",
        }
    }

    pub fn num_inputs(&self) -> usize {
        match self {
            Combinational::AndGate(_)
            | Combinational::Or(_)
            | Combinational::Xor(_)
            | Combinational::Xnor(_) => 2,
            Combinational::Not(_) | Combinational::Buf(_) => 1,
            Combinational::Mux2(_) => 3,
            _ => 0,
        }
    }

    /// Checks if this is a multi-input gate (AND/OR/XOR/XNOR).
    pub fn is_multi_input(&self) -> bool {
        self.num_inputs() > 1
    }

    /// Gets the primary output wire.
    pub fn output_wire(&self) -> Option<&Wire<S>> {
        self.y()
    }

    pub fn get_inputs(&self) -> Vec<Wire<S>> {
        match self {
            Combinational::AndGate(g) => vec![g.a.clone(), g.b.clone()],
            Combinational::Or(g) => vec![g.a.clone(), g.b.clone()],
            Combinational::Xor(g) => vec![g.a.clone(), g.b.clone()],
            Combinational::Xnor(g) => vec![g.a.clone(), g.b.clone()],
            Combinational::Not(g) => vec![g.a.clone()],
            Combinational::Buf(g) => vec![g.a.clone()],
            Combinational::Mux2(g) => vec![g.a.clone(), g.b.clone(), g.sel.clone()],
            _ => vec![],
        }
    }
}
