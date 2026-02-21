use crate::step2::HalfAdder;
use svql_query::prelude::*;
use svql_query_lib::OrGate;

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
