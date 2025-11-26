// use rstest::rstest;
// use std::sync::OnceLock;

// use svql_common::{ALL_TEST_CASES, Needle, TestCase};
// use svql_driver::Driver;

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
//         return Err("Invalid needle type for driver test".into());
//     };

//     let driver = Driver::new_workspace()?;

//     let needle_path = yosys_module.path().display().to_string();
//     let needle_module = yosys_module.module_name().to_string();
//     let haystack_path = tc.haystack.yosys_module.path().display().to_string();
//     let haystack_module = tc.haystack.yosys_module.module_name().to_string();

//     // Load pattern and haystack using the Driver
//     let (pattern_key, pattern_design_container) =
//         driver.get_or_load_design(&needle_path, &needle_module, &tc.config.needle_options)?;

//     let (haystack_key, haystack_design_container) = driver.get_or_load_design(
//         &haystack_path,
//         &haystack_module,
//         &tc.config.haystack_options,
//     )?;

//     // Create a context with both designs
//     let context = svql_driver::Context::new()
//         .with_design(pattern_key.clone(), pattern_design_container)
//         .with_design(haystack_key.clone(), haystack_design_container);

//     let pattern_design_container = context
//         .get(&pattern_key)
//         .expect("Pattern design not found in context");

//     let haystack_design_container = context
//         .get(&haystack_key)
//         .expect("Haystack design not found in context");

//     // Run subgraph search using the design instances
//     let embeddings = svql_subgraph::SubgraphMatcher::enumerate_with_indices(
//         pattern_design_container.design(),
//         haystack_design_container.design(),
//         pattern_design_container.index(),
//         haystack_design_container.index(),
//         &tc.config,
//     );

//     if embeddings.items.len() != tc.expected_matches {
//         return Err(format!(
//             "Driver test case '{}' failed: expected {} matches, got {}",
//             tc.name,
//             tc.expected_matches,
//             embeddings.items.len()
//         )
//         .into());
//     }

//     Ok(())
// }

// #[rstest]
// fn driver_all_netlist_cases() {
//     init_test_logger();

//     let results = netlist_cases()
//         .into_iter()
//         .map(run_case)
//         .collect::<Vec<_>>();

//     let failures: Vec<_> = results.into_iter().filter(|r| r.is_err()).collect();
//     if !failures.is_empty() {
//         let mut error_msg = format!("{} driver test cases failed", failures.len());
//         for failure in failures {
//             error_msg.push_str(&format!("\n - {}", failure.as_ref().unwrap_err()));
//         }
//         panic!("{}", error_msg);
//     }
// }

// #[test]
// fn driver_create_workspace() {
//     init_test_logger();
//     let d = Driver::new_workspace().expect("workspace driver");
//     // registry should be empty initially
//     assert_eq!(d.get_all_designs().len(), 0);
// }

// #[test]
// fn driver_create_context_missing_key() {
//     init_test_logger();
//     let d = Driver::new_workspace().expect("workspace driver");
//     // Make a key that won't be in the registry
//     let k = svql_driver::DriverKey::new("nonexistent.v", "missing_top".to_string());
//     let err = d.create_context(&[k]).unwrap_err();
//     match err {
//         svql_driver::DriverError::DesignLoading(msg) => assert!(msg.contains("Design not found")),
//         _ => panic!("unexpected error variant"),
//     }
// }
