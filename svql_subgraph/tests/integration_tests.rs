use rstest::rstest;
use std::sync::OnceLock;

use svql_common::{ALL_TEST_CASES, Pattern, TestCase};
use svql_subgraph::SubgraphMatcher;

fn init_test_logger() {
    static INIT: OnceLock<()> = OnceLock::new();
    let _ = INIT.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

fn netlist_cases() -> Vec<&'static TestCase> {
    ALL_TEST_CASES
        .iter()
        .filter(|tc| matches!(tc.pattern, Pattern::Netlist { .. }))
        .collect()
}

fn run_case(tc: &TestCase) {
    let Pattern::Netlist { yosys_module, .. } = tc.pattern else {
        return;
    };

    let needle = yosys_module
        .import_design(&tc.config.needle_options)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to import needle design for test case '{:#?}': {}",
                tc, e
            )
        });

    let haystack = tc
        .haystack
        .yosys_module
        .import_design(&tc.config.haystack_options)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to import haystack design for test case '{:#?}': {}",
                tc, e
            )
        });

    let embeddings = SubgraphMatcher::find_subgraphs(&needle, &haystack, &tc.config);

    assert!(
        embeddings.embeddings.len() == tc.expected_matches,
        "Subgraph test case '{}' failed: expected {} matches, got {}",
        tc.name,
        tc.expected_matches,
        embeddings.embeddings.len()
    );
}

#[rstest(
    tc,
    case("and_gate_self_dedupe_none"),
    case("and_gate_self_dedupe_all"),
    case("small_and_seq_3_and_tree_4_dedupe_none"),
    case("small_and_seq_3_and_tree_4_dedupe_all"),
    case("async_en_in_many_locked_regs"),
    case("async_mux_in_many_locked_regs"),
    case("sync_en_in_many_locked_regs"),
    case("sync_mux_in_many_locked_regs")
)]

fn subgraph_named_cases(tc: &str) {
    init_test_logger();
    let case = ALL_TEST_CASES
        .iter()
        .find(|t| t.name == tc)
        .expect("named case not found");
    run_case(case);
}
