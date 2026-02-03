use crate::query_test;
use svql_query::prelude::*;
use svql_query_lib::security::cwe1234::unlock_logic::UnlockLogic;

fn unlock_logic_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .dedupe(Dedupe::All)
}

query_test!(
    name: test_unlock_logic_simple,
    query: UnlockLogic,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_simple.v", "cwe1234_simple"),
    expect: 1,
    config: unlock_logic_config
);

// query_test!(
//     name: test_unlock_logic_deep,
//     query: UnlockLogic,
//     haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_deep.v", "cwe1234_deep"),
//     expect: usize::MAX,
//     config: unlock_logic_config
// );

// query_test!(
//     name: test_unlock_logic_swapped,
//     query: UnlockLogic,
//     haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_swapped.v", "cwe1234_swapped"),
//     expect: usize::MAX,
//     config: unlock_logic_config
// );

// query_test!(
//     name: test_unlock_logic_combined,
//     query: UnlockLogic,
//     haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_combined.v", "cwe1234_combined"),
//     expect: usize::MAX,
//     config: unlock_logic_config
// );

// query_test!(
//     name: test_unlock_logic_multi_reg,
//     query: UnlockLogic,
//     haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v", "cwe1234_multi_reg"),
//     expect: usize::MAX,
//     config: unlock_logic_config
// );
