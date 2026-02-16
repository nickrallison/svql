use svql_query::prelude::*;
use svql_query_lib::primitives::rec::RecAnd;
use svql_query_lib::primitives::rec::RecOr;

use crate::query_test;

const fn recursive_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .max_recursion_depth(Some(10))
}

const fn recursive_parametrized_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .max_recursion_depth(Some(10))
        .haystack_const_params(&[("N", "128")])
}

// --- Recursive AND Tests ---

query_test!(
    name: test_rec_and_small_tree,
    query: RecAnd,
    haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
    // (a&b) & (c&d) -> 3 AND gates total.
    expect: 3,
    config: recursive_config
);

query_test!(
    name: test_rec_and_large_tree,
    query: RecAnd,
    haystack: ("examples/fixtures/basic/and/verilog/and_tree.v", "and_tree"),
    // This is a generated tree.
    expect: 127,
    config: recursive_parametrized_config
);

query_test!(
    name: test_rec_and_mixed_tree,
    query: RecAnd,
    haystack: ("examples/fixtures/basic/and/json/mixed_and_tree.json", "mixed_and_tree"),
    // Contains ANDs mixed with MUX/NOR. Should find the pure AND chains.
    expect: 3,
    config: recursive_config
);

// --- Recursive OR Tests ---

query_test!(
    name: test_rec_or_wide_fanin,
    query: RecOr,
    haystack: ("examples/fixtures/composite/logic_tree/wide_fanin.v", "wide_fanin"),
    // wide_fanin is a | b | c | d.
    expect: 3,
    config: recursive_config
);

query_test!(
    name: test_rec_or_asymmetric,
    query: RecOr,
    haystack: ("examples/fixtures/composite/logic_tree/asymmetric_tree.v", "asymmetric_tree"),
    expect: 1,
    config: recursive_config
);

query_test!(
    name: test_rec_or_deep_3level,
    query: RecOr,
    haystack: ("examples/fixtures/composite/logic_tree/deep_3level.v", "deep_3level"),
    // Contains ORs at different levels.
    expect: 3,
    config: recursive_config
);
