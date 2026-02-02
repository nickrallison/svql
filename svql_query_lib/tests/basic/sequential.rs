#![allow(dead_code)]

use svql_query_lib::composites::dff_then_and::SdffeThenAnd;
use svql_query_lib::primitives::dff::*;

use crate::query_test;

// --- Shift Register / Sequence Tests ---

// Generate blocks don't work nicely
// query_test!(
//     name: test_seq_sdffe_shift_register,
//     query: Sdff,
//     haystack: ("examples/fixtures/basic/ff/verilog/seq_sdffe.v", "seq_sdffe"),
//     // FIFO_DEPTH = 8. Should find 8 SDFFEs.
//     expect: 8
// );

query_test!(
    name: test_seq_double_sdffe,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v", "seq_double_sdffe"),
    expect: 2
);

// --- Parallel / Double Tests ---

query_test!(
    name: test_par_double_sdffe,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/par_double_sdffe.v", "par_double_sdffe"),
    expect: 2
);

query_test!(
    name: test_and_q_double_sdffe,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v", "and_q_double_sdffe"),
    expect: 2
);

// --- Composite FF Tests ---

// Tests the SdffeThenAnd composite
query_test!(
    name: test_dff_then_and_composite,
    query: SdffeThenAnd,
    haystack: ("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v", "and_q_double_sdffe"),
    // The fixture has q1 & q2.
    // SdffeThenAnd looks for (DFF -> AND.a) AND (DFF -> AND.b).
    // Since q1 goes to AND and q2 goes to AND, but they are DIFFERENT DFFs,
    // this specific composite (which implies ONE dff feeding both inputs of an AND, or just connectivity?)
    expect: 0
);

// --- Feedback Loops & Inverters ---

query_test!(
    name: test_dff_loop_toggle,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/dff_loop_toggle.v", "dff_loop_toggle"),
    expect: 1
);

query_test!(
    name: test_dff_not,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/dff_not.v", "dff_not"),
    expect: 1
);

query_test!(
    name: test_not_dff,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/not_dff.v", "not_dff"),
    expect: 1
);
