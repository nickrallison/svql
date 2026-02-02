use svql_common::{ConfigBuilder, MatchLength};
use svql_subgraph::query_test;

fn cwe1234_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(svql_common::Dedupe::Inner)
}

query_test!(
    name: test_adffe_mux_enable_multi_width,
    needle: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    expect: 5,
    config: cwe1234_config
);
