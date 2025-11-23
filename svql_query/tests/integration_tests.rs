// svql_query/tests/integration_tests.rs
use rstest::rstest;
use std::sync::OnceLock;
use svql_query::composites::rec_or::RecOr;
use svql_query::primitives::and::AndGate;
use svql_query::primitives::not::NotGate;
use svql_query::traits::composite::SearchableComposite;
use svql_query::traits::variant::SearchableVariant;

use svql_common::{ALL_TEST_CASES, Needle, TestCase};
use svql_driver::Driver;
use svql_query::composites::dff_then_and::SdffeThenAnd;
use svql_query::composites::rec_and::RecAnd;
use svql_query::variants::and_any::AndAny;
use svql_query::traits::netlist::SearchableNetlist;
use svql_query::{Search, instance::Instance};

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
        "svql_query::queries::netlist::basic::not::NotGate" => {
            // ADDED: NotGate context arm
            <NotGate<Search> as SearchableNetlist>::context(&driver, &tc.config.needle_options)
        }
        "svql_query::queries::composites::dff_then_and::SdffeThenAnd" => {
            <SdffeThenAnd<Search> as SearchableComposite>::context(
                &driver,
                &tc.config.needle_options,
            )
        }
        "svql_query::variants::and_any::AndAny" => {
            <AndAny<Search> as SearchableVariant>::context(&driver, &tc.config.needle_options)
        }
        "svql_query::queries::composites::rec_and::RecAnd" => {
            <RecAnd<Search> as SearchableComposite>::context(&driver, &tc.config.needle_options)
        }
        "svql_query::queries::composites::rec_or::RecOr" => {
            <RecOr<Search> as SearchableComposite>::context(&driver, &tc.config.needle_options)
        }
        "svql_query::queries::security::cwe1234::unlock_logic::UnlockLogic" => {
            todo!()
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
        "svql_query::queries::netlist::basic::not::NotGate" => {
            // ADDED: NotGate query arm
            <NotGate<Search> as SearchableNetlist>::query(&hk, &ctx, root.clone(), &tc.config).len()
        }
        "svql_query::queries::composites::dff_then_and::SdffeThenAnd" => {
            <SdffeThenAnd<Search> as SearchableComposite>::query(
                &hk,
                &ctx,
                root.clone(),
                &tc.config,
            )
            .len()
        }
        "svql_query::variants::and_any::AndAny" => {
            <AndAny<Search> as SearchableVariant>::query(&hk, &ctx, root.clone(), &tc.config)
                .len()
        }
        "svql_query::queries::composites::rec_and::RecAnd" => {
            <RecAnd<Search> as SearchableComposite>::query(&hk, &ctx, root.clone(), &tc.config)
                .len()
        }
        "svql_query::queries::composites::rec_or::RecOr" => {
            <RecOr<Search> as SearchableComposite>::query(&hk, &ctx, root.clone(), &tc.config).len()
        }
        "svql_query::queries::security::cwe1234::unlock_logic::UnlockLogic" => {
            todo!()
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
