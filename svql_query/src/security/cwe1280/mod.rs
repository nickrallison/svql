use crate::security::primitives::uninit_reg::{UninitReg, UninitRegEn};
use svql_macros::enum_composite;

composite! {
    name: Cwe1280,
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
