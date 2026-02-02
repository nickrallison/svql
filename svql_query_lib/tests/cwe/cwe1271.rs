use crate::query_test;
use svql_query::prelude::*;
use svql_query_lib::security::cwe1271::Cwe1271;

fn cwe1271_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
}

query_test!(
    name: test_cwe1271_simple,
    query: Cwe1271,
    haystack: ("examples/fixtures/cwes/cwe1271/cwe1271_simple.v", "cwe1271_simple"),
    expect: 1,
    config: cwe1271_config
);

query_test!(
    name: test_cwe1271_en,
    query: Cwe1271,
    haystack: ("examples/fixtures/cwes/cwe1271/cwe1271_en.v", "cwe1271_en"),
    expect: 1,
    config: cwe1271_config
);

query_test!(
    name: test_cwe1271_fixed,
    query: Cwe1271,
    haystack: ("examples/fixtures/cwes/cwe1271/cwe1271_fixed.v", "cwe1271_fixed"),
    expect: 0,
    config: cwe1271_config
);

query_test!(
    name: test_cwe1271_multi,
    query: Cwe1271,
    haystack: ("examples/fixtures/cwes/cwe1271/cwe1271_multi.v", "cwe1271_multi"),
    expect: 3,
    config: cwe1271_config
);
