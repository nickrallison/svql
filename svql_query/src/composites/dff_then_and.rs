use crate::instance::Instance;
use crate::primitives::and::AndGate;
use crate::primitives::dff::Sdffe;
use crate::traits::composite::{Composite, MatchedComposite};
use crate::traits::netlist::SearchableNetlist;
use svql_macros::composite;

// Generate everything: struct, impls, query, etc.
composite! {
    name: SdffeThenAnd,
    subs: [
        sdffe: Sdffe,
        and_gate: AndGate
    ],
    connections: [
        [
            sdffe . q => and_gate . a,
            sdffe . q => and_gate . b
        ]
    ]
}
