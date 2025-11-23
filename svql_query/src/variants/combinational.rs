// svql_query/src/variants/combinational.rs
// Variant for detecting various combinational logic gates.
// Covers basic gates: AND variants, OR, XOR, NOT, BUF, MUX2.
// Useful for analyzing pure combinational paths (no state/flops).

use crate::primitives::and::AndGate;
use crate::primitives::buf::BufGate;
use crate::primitives::mux2::Mux2Gate;
use crate::primitives::not::NotGate;
use crate::primitives::or::OrGate;
use crate::primitives::xnor::XnorGate;
use crate::primitives::xor::XorGate;
use crate::{State, Wire};
use svql_macros::variant;

// Variant for various combinational gates.
// Matches any of the specified gate types in the netlist.
//
// # Common Ports
// - `input_a`: Primary input (always present; for single-input gates like NOT/BUF).
// - `input_b`: Secondary input (for dual-input gates like AND/OR/XOR; optional).
// - `output`: The gate's output wire.
variant! {
    name: Combinational,
    variants: [
        // AND variants (multi-style implementations)
        (AndGate, "and_gate", AndGate),

        // OR gate
        (Or, "or_gate", OrGate),

        // XOR/XNOR gates
        (Xor, "xor_gate", XorGate),
        (Xnor, "xnor_gate", XnorGate),

        // Single-input gates
        (Not, "not_gate", NotGate),
        (Buf, "buf_gate", BufGate),

        // MUX (conditional combinational)
        (Mux2, "mux2_gate", Mux2Gate)
    ],
    common_ports: {
        y: "y"
    }
}

impl<S> Combinational<S>
where
    S: State,
{
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
        }
    }

    /// Checks if this is a multi-input gate (AND/OR/XOR/XNOR).
    pub fn is_multi_input(&self) -> bool {
        self.num_inputs() > 1
    }

    /// Gets the primary output wire.
    pub fn output_wire(&self) -> &Wire<S> {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Search;
    use crate::instance::Instance;

    // Mock a simple AND gate for testing (in real use, this would come from a query hit)
    fn mock_and_gate() -> Combinational<Search> {
        let path = Instance::root("mock_and".to_string());
        Combinational::AndGate(AndGate::new(path.child("and".to_string())))
    }

    // Mock a NOT gate
    fn mock_not_gate() -> Combinational<Search> {
        let path = Instance::root("mock_not".to_string());
        Combinational::Not(NotGate::new(path.child("not".to_string())))
    }

    #[test]
    fn test_gate_type() {
        assert_eq!(mock_and_gate().gate_type(), "AND Gate");
        assert_eq!(mock_not_gate().gate_type(), "NOT Gate");
    }

    #[test]
    fn test_num_inputs() {
        assert_eq!(mock_and_gate().num_inputs(), 2);
        assert_eq!(mock_not_gate().num_inputs(), 1);
    }

    #[test]
    fn test_is_multi_input() {
        assert!(mock_and_gate().is_multi_input());
        assert!(!mock_not_gate().is_multi_input());
    }

    #[test]
    fn test_output_wire() {
        let and = mock_and_gate();
        let output = and.output_wire();
        assert_eq!(output.path.inst_path(), "mock_and.and.y"); // Assuming AndGate.y path
    }

    #[test]
    fn test_secondary_input() {
        let and = mock_and_gate();
        assert!(and.get_inputs().get(1).is_some());

        let not = mock_not_gate();
        assert!(not.get_inputs().get(1).is_none());
    }
}
