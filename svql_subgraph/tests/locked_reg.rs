use svql_common::{ConfigBuilder, MatchLength};
use svql_subgraph::query_test;

fn cwe1234_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
}

query_test!(
    name: test_adffe_mux_enable_multi_width,
    needle: ("examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il", "async_mux"),
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    expect: 5,
    config: cwe1234_config
);
