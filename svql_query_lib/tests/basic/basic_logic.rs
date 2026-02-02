use svql_query::prelude::*;

use crate::query_test;

// --- Define Missing Needles ---

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/basic/and/verilog/and_2_seq.v",
    module = "and_2_seq"
)]
pub struct And2Seq {
    #[port(input)]
    pub a: Wire,
    #[port(input)]
    pub b: Wire,
    #[port(input)]
    pub c: Wire,
    #[port(output)]
    pub y: Wire,
}

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/basic/or/verilog/or_2_seq.v",
    module = "or_2_seq"
)]
pub struct Or2Seq {
    #[port(input)]
    pub a: Wire,
    #[port(input)]
    pub b: Wire,
    #[port(input)]
    pub c: Wire,
    #[port(output)]
    pub y: Wire,
}

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/basic/not/verilog/not_2_seq.v",
    module = "not_2_seq"
)]
pub struct Not2Seq {
    #[port(input)]
    pub a: Wire,
    #[port(output)]
    pub y: Wire,
}

// --- Tests ---

query_test!(
    name: test_and_2_seq,
    query: And2Seq,
    haystack: ("examples/fixtures/basic/and/verilog/small_and_seq.v", "small_and_seq"),
    expect: 1
);

// Generates Broken
// query_test!(
//     name: test_and_seq_generator,
//     query: And2Seq,
//     haystack: ("examples/fixtures/basic/and/verilog/and_seq.v", "and_seq"),
//     expect: 1
// );

query_test!(
    name: test_or_2_seq,
    query: Or2Seq,
    haystack: ("examples/fixtures/composite/logic_tree/simple_2level.v", "simple_2level"),
    // simple_2level is (a|b) & (c|d). It does NOT contain (a|b)|c.
    // We expect 0 here, verifying it doesn't false positive on a tree structure.
    expect: 0
);
