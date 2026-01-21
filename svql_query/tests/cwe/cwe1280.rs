use crate::query_test;
use svql_query::prelude::*;
use svql_query::security::cwe1280::Cwe1280;
use svql_query::security::cwe1280::grant_access::GrantAccess;

#[allow(dead_code)]
/// Configuration for the full CWE-1280 structural pattern
fn cwe1280_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::NeedleSubsetHaystack)
        .pattern_vars_match_design_consts(true)
        .haystack_opt(true)
        .dedupe(Dedupe::None)
}

/// Configuration for the GrantAccess atomic netlist component
fn grant_access_config(c: ConfigBuilder) -> ConfigBuilder {
    c.match_length(MatchLength::First)
        .dedupe(Dedupe::None)
        .pattern_vars_match_design_consts(true)
}

// --- Full Pattern Tests ---

// query_test!(
//     name: test_cwe1280_fixed,
//     query: Cwe1280<Search>,
//     haystack: ("examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v", "cwe1280_fixed"),
//     expect: 0,
//     config: cwe1280_config
// );

// query_test!(
//     name: test_cwe1280_vuln,
//     query: Cwe1280<Search>,
//     haystack: ("examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v", "cwe1280_vuln"),
//     expect: 1,
//     config: cwe1280_config
// );

// --- Sub-component (GrantAccess) Tests ---

query_test!(
    name: test_grant_access_fixed,
    query: GrantAccess<Search>,
    haystack: ("examples/fixtures/cwes/cwe1280/verilog/cwe1280_fixed.v", "cwe1280_fixed"),
    expect: 1,
    config: grant_access_config
);

query_test!(
    name: test_grant_access_vuln,
    query: GrantAccess<Search>,
    haystack: ("examples/fixtures/cwes/cwe1280/verilog/cwe1280_vuln.v", "cwe1280_vuln"),
    expect: 1,
    config: grant_access_config
);
