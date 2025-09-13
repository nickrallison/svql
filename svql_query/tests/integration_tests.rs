use rstest::rstest;
use std::sync::OnceLock;

use svql_common::{ALL_TEST_CASES, Needle, TestCase};
use svql_driver::Driver;

// Include the generated dispatch functions
mod gen_dispatch {
    include!(concat!(env!("OUT_DIR"), "/svql_query_query_dispatch.rs"));
}

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
        .filter(|tc| match tc.needle {
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

    let query_name = match tc.needle {
        Needle::Netlist {
            pattern_query_type: Some(name),
            ..
        } => name,
        Needle::Composite {
            pattern_query_type: name,
        } => name,
        _ => return Err("Invalid needle type for query test".into()),
    };

    let haystack_path = &tc.haystack.yosys_module.path().display().to_string();
    let haystack_module = tc.haystack.yosys_module.module_name();

    // Use the generated dispatch function
    match gen_dispatch::run_count_for_type_name(
        query_name,
        &driver,
        haystack_path,
        haystack_module,
        &tc.config,
    ) {
        Ok(actual) => {
            if actual != tc.expected_matches {
                return Err(format!(
                    "Query test case '{}' failed: expected {} matches, got {}",
                    tc.name, tc.expected_matches, actual
                )
                .into());
            }
        }
        Err(e) => return Err(format!("Query test case '{}' failed: {}", tc.name, e).into()),
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

#[test]
fn query_known_types_list() {
    init_test_logger();
    let known = gen_dispatch::known_query_type_names();
    assert!(!known.is_empty(), "Should have discovered some query types");

    // Print for debugging
    for name in known {
        println!("Known query type: {}", name);
    }
}
