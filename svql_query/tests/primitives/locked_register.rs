use crate::query_test;
use svql_query::prelude::*;
use svql_query::security::primitives::locked_register::LockedRegister;

// Case 1: Simple (NeedleSubsetHaystack)
query_test!(
    name: test_locked_reg_simple,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_simple.v", "cwe1234_simple"),
    expect: 1,
    config: |c| c.match_length(MatchLength::NeedleSubsetHaystack).dedupe(Dedupe::All)
);

// Remaining cases use MatchLength::Exact

query_test!(
    name: test_locked_reg_multi,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_reg.v", "cwe1234_multi_reg"),
    expect: 3,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_deep,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_deep.v", "cwe1234_deep"),
    expect: 1,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_fixed,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_fixed.v", "cwe1234_fixed"),
    expect: 0,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_sync_reset,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_sync_reset.v", "cwe1234_sync_reset"),
    expect: 1,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_enabled,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_enabled.v", "cwe1234_enabled"),
    expect: 1,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_wide,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_wide_reg.v", "cwe1234_wide_reg"),
    expect: 1,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_mixed_resets,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_mixed_resets.v", "cwe1234_mixed_resets"),
    expect: 2,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_no_reset,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_no_reset.v", "cwe1234_no_reset"),
    expect: 1,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);

query_test!(
    name: test_locked_reg_multi_width,
    query: LockedRegister<Search>,
    haystack: ("examples/fixtures/cwes/cwe1234/cwe1234_multi_width.v", "cwe1234_multi_width"),
    expect: 1,
    config: |c| c.match_length(MatchLength::Exact).dedupe(Dedupe::All)
);
