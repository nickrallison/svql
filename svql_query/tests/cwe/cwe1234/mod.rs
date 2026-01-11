mod unlock_logic;

use crate::query_test;
use svql_query::prelude::*;
use svql_query::security::cwe1234::unlock_logic::UnlockLogic;

// Common config for CWE-1234 tests
fn cwe1234_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
}

query_test!(
    name: test_cwe1234_simple,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_simple.v", "cwe1234_simple"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_swapped,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_swapped.v", "cwe1234_swapped"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_sync_reset,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_sync_reset.v", "cwe1234_sync_reset"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_wide_reg,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v", "cwe1234_wide_reg"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_not_deep,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_deep.v", "cwe1234_not_deep"),
    expect: 3,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_not_positions,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_positions.v", "cwe1234_not_positions"),
    expect: 5,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_not_right,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_right.v", "cwe1234_not_right"),
    expect: 3,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_not_alternating,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_alternating.v", "cwe1234_not_alternating"),
    expect: 4,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_multi_reg,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v", "cwe1234_multi_reg"),
    expect: 3,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_multi_width,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    expect: 4,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_no_reset,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v", "cwe1234_no_reset"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_mixed_resets,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v", "cwe1234_mixed_resets"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_combined,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_combined.v", "cwe1234_combined"),
    expect: 0,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_deep,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_deep.v", "cwe1234_deep"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_enabled,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_enabled.v", "cwe1234_enabled"),
    expect: 1,
    config: cwe1234_config
);

query_test!(
    name: test_cwe1234_fixed,
    query: UnlockLogic<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_fixed.v", "cwe1234_fixed"),
    expect: 0,
    config: cwe1234_config
);
