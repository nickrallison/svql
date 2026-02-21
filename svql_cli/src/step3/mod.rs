use crate::step2::HalfAdder;
use svql_query::prelude::*;
use svql_query_lib::OrGate;

// # Task: Define `FullAdderComposite`.
// 1. Define `FullAdderComposite`.
// 2. Instantiate two `HalfAdder` submodules (`ha1`, `ha2`) and one `OrGate` (`final_or`).
// 3. Use `#[connection]` to link `ha1.sum` to `ha2.a`.
// 4. Use `#[connection]` to link `ha1.carry` to `final_or.a` and `ha2.carry` to `final_or.b`.
// 5. Use `#[alias]` to expose the top-level ports: `a`, `b`, `cin`, `sum`, and `cout`.

#[derive(Debug, Clone, Composite)]
#[connection(from = ["ha1", "sum"], to = ["ha2", "a"])]
#[connection(from = ["ha1", "carry"], to = ["final_or", "a"])]
#[connection(from = ["ha2", "carry"], to = ["final_or", "b"])]
pub struct FullAdderComposite {
    #[submodule]
    pub ha1: HalfAdder,
    #[submodule]
    pub ha2: HalfAdder,
    #[submodule]
    pub final_or: OrGate,

    #[alias(input, target = ["ha1", "a"])]
    pub a: Wire,
    #[alias(input, target = ["ha1", "b"])]
    pub b: Wire,
    #[alias(input, target = ["ha2", "b"])]
    pub cin: Wire,
    #[alias(output, target = ["ha2", "sum"])]
    pub sum: Wire,
    #[alias(output, target = ["final_or", "y"])]
    pub cout: Wire,
}
