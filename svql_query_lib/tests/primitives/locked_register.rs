use crate::query_test;
use svql_query::prelude::*;
use svql_query_lib::security::primitives::locked_register::{AsyncDffMuxEnable, LockedRegister};

// Common config for CWE-1234 tests
fn cwe1234_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
}

query_test!(
    name: test_locked_reg_simple,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_simple.v", "cwe1234_simple"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_swapped,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_swapped.v", "cwe1234_swapped"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_sync_reset,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_sync_reset.v", "cwe1234_sync_reset"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_wide_reg,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v", "cwe1234_wide_reg"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_not_deep,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_deep.v", "cwe1234_not_deep"),
    expect: 6,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_not_positions,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_positions.v", "cwe1234_not_positions"),
    expect: 8,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_not_right,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_right.v", "cwe1234_not_right"),
    expect: 6,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_not_alternating,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_not_alternating.v", "cwe1234_not_alternating"),
    expect: 4,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_multi_reg,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v", "cwe1234_multi_reg"),
    expect: 6,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_multi_width,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    expect: 5,
    config: cwe1234_config
);

// query_test!(
//     name: test_locked_reg_no_reset,
//     query: LockedRegister,
//     haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v", "cwe1234_no_reset"),
//     expect: 1,
//     config: cwe1234_config
// );

query_test!(
    name: test_locked_reg_mixed_resets,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v", "cwe1234_mixed_resets"),
    expect: 4,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_combined,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_combined.v", "cwe1234_combined"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_deep,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_deep.v", "cwe1234_deep"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_enabled,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_enabled.v", "cwe1234_enabled"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_locked_reg_fixed,
    query: LockedRegister,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_fixed.v", "cwe1234_fixed"),
    expect: 2,
    config: cwe1234_config
);

query_test!(
    name: test_adffe_mux_enable_multi_width,
    query: AsyncDffMuxEnable,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    expect: 5,
    config: cwe1234_config
);
