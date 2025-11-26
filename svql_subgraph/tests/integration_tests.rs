// use rstest::rstest;
// use std::sync::OnceLock;

// use svql_common::{ALL_TEST_CASES, Needle, TestCase};
// use svql_subgraph::SubgraphMatcher;

// fn init_test_logger() {
//     static INIT: OnceLock<()> = OnceLock::new();
//     let _ = INIT.get_or_init(|| {
//         let _ = tracing_subscriber::fmt()
//             .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//             .with_test_writer()
//             .try_init();
//     });
// }

// fn netlist_cases() -> Vec<&'static TestCase> {
//     ALL_TEST_CASES
//         .iter()
//         .filter(|tc| matches!(tc.needle, Needle::Netlist { .. }))
//         .collect()
// }

// fn run_case(tc: &TestCase) -> Result<(), Box<dyn std::error::Error>> {
//     let Needle::Netlist { yosys_module, .. } = tc.needle else {
//         return Err("Invalid needle".into());
//     };

//     let needle = yosys_module.import_design(&tc.config.needle_options)?;

//     let haystack = tc
//         .haystack
//         .yosys_module
//         .import_design(&tc.config.haystack_options)?;

//     let embeddings = SubgraphMatcher::enumerate_all(&needle, &haystack, &tc.config);

//     if embeddings.items.len() != tc.expected_matches {
//         return Err(format!(
//             "Subgraph test case '{}' failed: expected {} matches, got {}",
//             tc.name,
//             tc.expected_matches,
//             embeddings.items.len()
//         )
//         .into());
//     }
//     Ok(())
// }

// #[rstest]
// fn subgraph_all_netlist_cases() {
//     init_test_logger();

//     let results = netlist_cases()
//         .into_iter()
//         .map(run_case)
//         .collect::<Vec<_>>();

//     let failures: Vec<_> = results.into_iter().filter(|r| r.is_err()).collect();
//     if !failures.is_empty() {
//         let mut error_msg = format!("{} subgraph test cases failed", failures.len());
//         for failure in failures {
//             error_msg.push_str(&format!("\n - {}", failure.as_ref().unwrap_err()));
//         }
//         panic!("{}", error_msg);
//     }
// }
