use rstest::rstest;
use std::sync::OnceLock;

use svql_common::{ALL_TEST_CASES, Needle, TestCase};
use svql_driver::Driver;
use svql_query::{
    Search,
    composite::{SearchableComposite, SearchableEnumComposite},
    instance::Instance,
    netlist::SearchableNetlist,
    queries::{
        composite::dff_then_and::{SdffeThenAnd, SdffeThenAnd2},
        enum_composite::and_any::AndAny,
        netlist::basic::and::AndGate,
    },
};

// No generated dispatch—manual match for test cases (mirrors direct SubgraphMatcher call in subgraph tests)

fn init_test_logger() {
    static INIT: OnceLock<()> = OnceLock::new();
    let _ = INIT.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    });
}

fn query_cases() -> Vec<&'static TestCase> {
    ALL_TEST_CASES
        .iter()
        .filter(|tc| match &tc.needle {
            Needle::Netlist {
                pattern_query_type: Some(_),
                ..
            } => true,
            Needle::Composite { .. } => true,
            _ => false,
        })
        .collect()
}

fn run_case(tc: &TestCase) -> Result<(), Box<dyn std::error::Error>> {
    let driver = Driver::new_workspace()?;

    // FIXED: Use ref patterns to bind name as &str (avoids &&str from matching on &tc.needle)
    let query_name: &str = match &tc.needle {
        Needle::Netlist {
            pattern_query_type: Some(name),
            ..
        } => name,
        Needle::Composite {
            pattern_query_type: name,
        } => name,
        _ => return Err("Invalid needle type for query test".into()),
    };

    let haystack_path = tc.haystack.yosys_module.path().display().to_string();
    let haystack_module = tc.haystack.yosys_module.module_name().to_string();

    // Load haystack (shared for all)
    let (hk, hd) = driver
        .get_or_load_design(
            &haystack_path,
            &haystack_module,
            &tc.config.haystack_options,
        )
        .map_err(|e| format!("Failed to load haystack: {}", e))?;
    let root = Instance::root("test_root".to_string());

    // Build context (needle-specific; merges for composites/enums)
    // Traits in scope, so context/query calls work
    let ctx = match query_name {
        "svql_query::queries::netlist::basic::and::AndGate" => {
            <AndGate<Search> as SearchableNetlist>::context(&driver, &tc.config.needle_options)
        }
        "svql_query::queries::composite::dff_then_and::SdffeThenAnd" => {
            <SdffeThenAnd<Search> as SearchableComposite>::context(
                &driver,
                &tc.config.needle_options,
            )
        }
        "svql_query::queries::composite::dff_then_and::SdffeThenAnd2" => {
            // Arm for macro-generated composite (verifies macro)
            <SdffeThenAnd2<Search> as SearchableComposite>::context(
                &driver,
                &tc.config.needle_options,
            )
        }
        "svql_query::queries::enum_composite::and_any::AndAny" => {
            <AndAny<Search> as SearchableEnumComposite>::context(&driver, &tc.config.needle_options)
        }
        _ => return Err(format!("No context handler for query type: {}", query_name).into()),
    }
    .map_err(|e| format!("Failed to build context for {}: {}", query_name, e))?;

    let ctx = ctx.with_design(hk.clone(), hd);

    // FIXED: Compute hit_count directly in match arms (avoids incompatible Vec<T> types for hits)
    // Each arm runs query and returns len() (common usize type)
    let hit_count = match query_name {
        "svql_query::queries::netlist::basic::and::AndGate" => {
            <AndGate<Search> as SearchableNetlist>::query(&hk, &ctx, root.clone(), &tc.config).len()
        }
        "svql_query::queries::composite::dff_then_and::SdffeThenAnd" => {
            <SdffeThenAnd<Search> as SearchableComposite>::query(
                &hk,
                &ctx,
                root.clone(),
                &tc.config,
            )
            .len()
        }
        "svql_query::queries::composite::dff_then_and::SdffeThenAnd2" => {
            // Arm for macro-generated composite (verifies macro)
            <SdffeThenAnd2<Search> as SearchableComposite>::query(
                &hk,
                &ctx,
                root.clone(),
                &tc.config,
            )
            .len()
        }
        "svql_query::queries::enum_composite::and_any::AndAny" => {
            <AndAny<Search> as SearchableEnumComposite>::query(&hk, &ctx, root.clone(), &tc.config)
                .len()
        }
        _ => return Err(format!("No query handler for query type: {}", query_name).into()),
    };

    if hit_count != tc.expected_matches {
        return Err(format!(
            "Query test case '{}' failed: expected {} matches, got {}",
            tc.name, tc.expected_matches, hit_count
        )
        .into());
    }

    Ok(())
}

#[rstest]
fn query_all_cases() {
    init_test_logger();

    let results = query_cases().into_iter().map(run_case).collect::<Vec<_>>();

    let failures: Vec<_> = results.into_iter().filter(|r| r.is_err()).collect();
    if !failures.is_empty() {
        let mut error_msg = format!("{} query test cases failed", failures.len());
        for failure in failures {
            error_msg.push_str(&format!("\n - {}", failure.as_ref().unwrap_err()));
        }
        panic!("{}", error_msg);
    }
}
