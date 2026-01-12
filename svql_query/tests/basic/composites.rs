use svql_query::composites::rec_and::RecAnd;
use svql_query::composites::rec_or::RecOr;
use svql_query::prelude::*;

use crate::query_test;

fn recursive_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
        .max_recursion_depth(Some(10))
}

// --- Recursive AND Tests ---

query_test!(
    name: test_rec_and_small_tree,
    query: RecAnd<Search>,
    haystack: ("examples/fixtures/basic/and/verilog/small_and_tree.v", "small_and_tree"),
    // (a&b) & (c&d) -> 3 AND gates total.
    // RecAnd matches the root AND, plus the two children.
    // Depending on implementation, it might return the root match or all sub-matches.
    // Assuming it returns the root of the tree:
    expect: 1,
    config: recursive_config
);

query_test!(
    name: test_rec_and_large_tree,
    query: RecAnd<Search>,
    haystack: ("examples/fixtures/basic/and/verilog/and_tree.v", "and_tree"),
    // This is a generated tree. We expect at least 1 large tree match.
    expect: 1,
    config: recursive_config
);

query_test!(
    name: test_rec_and_mixed_tree,
    query: RecAnd<Search>,
    haystack: ("examples/fixtures/basic/and/verilog/mixed_and_tree.v", "mixed_and_tree"),
    // Contains ANDs mixed with MUX/NOR. Should find the pure AND chains.
    expect: 1,
    config: recursive_config
);

// --- Recursive OR Tests ---

query_test!(
    name: test_rec_or_wide_fanin,
    query: RecOr<Search>,
    haystack: ("examples/fixtures/composite/logic_tree/wide_fanin.v", "wide_fanin"),
    // wide_fanin is a | b | c | d.
    expect: 1,
    config: recursive_config
);

query_test!(
    name: test_rec_or_asymmetric,
    query: RecOr<Search>,
    haystack: ("examples/fixtures/composite/logic_tree/asymmetric_tree.v", "asymmetric_tree"),
    // Contains (a&b) & (c&d) -> AND tree
    // Then (AND_LEFT | (e&f)).
    // RecOr should find the top level OR.
    expect: 1,
    config: recursive_config
);

query_test!(
    name: test_rec_or_deep_3level,
    query: RecOr<Search>,
    haystack: ("examples/fixtures/composite/logic_tree/deep_3level.v", "deep_3level"),
    // Contains ORs at different levels.
    expect: 1,
    config: recursive_config
);
