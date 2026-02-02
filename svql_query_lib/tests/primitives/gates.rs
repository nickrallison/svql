use crate::query_test;
use svql_query_lib::primitives::dff::*;
use svql_query_lib::primitives::gates::*;

query_test!(
    name: test_and_gate_scan,
    query: AndGate,
    haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
    expect: 3
);

query_test!(
    name: test_dff_gate_scan,
    query: DffAny,
    haystack: ("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v", "seq_double_sdffe"),
    expect: 2
);

query_test!(
    name: test_mux_gate_scan,
    query: MuxGate,
    haystack: ("examples/fixtures/composite/logic_tree/mux_tree.v", "mux_tree"),
    expect: 3
);

query_test!(
    name: test_xor_chain_scan,
    query: XorGate,
    haystack: ("examples/fixtures/composite/logic_tree/xor_chain.v", "xor_chain"),
    expect: 3
);

query_test!(
    name: test_eq_gate_scan,
    query: EqGate,
    haystack: ("examples/patterns/security/access_control/grant_access/verilog/grant_access.v", "grant_access"),
    expect: 1
);

// --- Mixed Gates Tests ---

query_test!(
    name: test_mixed_gates_and,
    query: AndGate,
    haystack: ("examples/fixtures/composite/logic_tree/mixed_gates.v", "mixed_gates"),
    expect: 1
);

query_test!(
    name: test_mixed_gates_xor,
    query: XorGate,
    haystack: ("examples/fixtures/composite/logic_tree/mixed_gates.v", "mixed_gates"),
    expect: 1
);

query_test!(
    name: test_mixed_gates_not,
    query: NotGate,
    haystack: ("examples/fixtures/composite/logic_tree/mixed_gates.v", "mixed_gates"),
    expect: 1
);

query_test!(
    name: test_mixed_gates_or,
    query: OrGate,
    haystack: ("examples/fixtures/composite/logic_tree/mixed_gates.v", "mixed_gates"),
    expect: 2
);

// --- Complex Comparison Tests ---

query_test!(
    name: test_complex_comparison_eq,
    query: EqGate,
    haystack: ("examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v", "cwe1280_fixed"),
    expect: 1
);

query_test!(
    name: test_complex_comparison_mux,
    query: MuxGate,
    haystack: ("examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v", "cwe1280_fixed"),
    expect: 2
);
