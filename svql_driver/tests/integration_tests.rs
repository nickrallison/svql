#[cfg(test)]
mod tests {
    use svql_common::{ALL_TEST_CASES, Pattern};
    use svql_driver::Driver;

    #[test]
    fn test_all_driver_cases() {
        let driver = Driver::new_workspace().expect("Failed to create driver");

        let test_cases = ALL_TEST_CASES.iter().filter(|tc| tc.pattern.is_netlist());
        let test_results: Vec<Result<(), Box<dyn std::error::Error>>> = test_cases
            .map(|test_case| run_driver_test_case(&driver, test_case))
            .collect();

        let num_errors = test_results.iter().filter(|r| r.is_err()).count();

        if num_errors > 0 {
            eprintln!(
                "{} out of {} driver tests failed",
                num_errors,
                test_results.len()
            );
            for result in test_results {
                match result {
                    Ok(()) => {}
                    Err(e) => eprintln!("Driver test failed: {}", e),
                }
            }
            assert!(false, "Some driver test cases failed");
        }
    }

    fn run_driver_test_case(
        driver: &Driver,
        test_case: &svql_common::TestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (netlist_pattern_path, netlist_pattern_module) = match test_case.pattern {
            Pattern::Netlist { path, module, .. } => (path, module),
            _ => panic!("Test case is not a netlist pattern"),
        };

        // Load pattern using driver
        let (pattern_key, pattern_design) = driver
            .get_or_load_design(netlist_pattern_path, netlist_pattern_module.to_string())
            .expect(&format!("Failed to load pattern: {}", netlist_pattern_path));

        // Load haystack using driver
        let (haystack_key, haystack_design) = driver
            .get_or_load_design(
                test_case.haystack.path,
                test_case.haystack.module.to_string(),
            )
            .expect(&format!(
                "Failed to load haystack: {}",
                test_case.haystack.path
            ));

        // Create context with both designs
        let context = driver
            .create_context(&[pattern_key, haystack_key])
            .expect("Failed to create context");

        // Run subgraph search using the context
        let matches = svql_subgraph::find_subgraphs(
            pattern_design.as_ref(),
            haystack_design.as_ref(),
            &test_case.config,
        );

        let expected_matches = test_case.expected_matches;
        let actual_matches = matches.len();

        if actual_matches != expected_matches {
            return Err(format!(
                "Driver test case '{}' failed: expected {} matches, got {}",
                test_case.name, expected_matches, actual_matches
            )
            .into());
        }

        Ok(())
    }
}
