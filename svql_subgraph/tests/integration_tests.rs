#[cfg(test)]
mod tests {
    use log::trace;
    use svql_common::{ALL_TEST_CASES, Pattern, import_design};

    #[test]
    fn test_all_netlist_cases() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .try_init()
            .expect("Failed to initialize logger");
        let test_cases = ALL_TEST_CASES.iter().filter(|tc| tc.pattern.is_netlist());
        let test_results: Vec<Result<(), Box<dyn std::error::Error>>> = test_cases
            .map(|test_case| run_test_case(test_case))
            .collect();

        let num_errors = test_results.iter().filter(|r| r.is_err()).count();

        if num_errors > 0 {
            eprintln!("{} out of {} tests failed", num_errors, test_results.len());
            for result in test_results {
                match result {
                    Ok(()) => {}
                    Err(e) => eprintln!("Test failed: {}", e),
                }
            }
            assert!(false, "Some test cases failed");
        }
    }

    fn run_test_case(test_case: &svql_common::TestCase) -> Result<(), Box<dyn std::error::Error>> {
        let (netlist_pattern_path, netlist_pattern_module) = match test_case.pattern {
            Pattern::Netlist { path, module, .. } => (path, module),
            _ => panic!("Test case is not a netlist pattern"),
        };

        let needle = import_design(netlist_pattern_path.into(), netlist_pattern_module)
            .expect(&format!("Failed to load needle: {}", netlist_pattern_path));
        let haystack =
            import_design(test_case.haystack.path.into(), test_case.haystack.module).expect(
                &format!("Failed to load haystack: {}", test_case.haystack.path),
            );

        let matches = svql_subgraph::find_subgraphs(&needle, &haystack, &test_case.config);

        let expected_matches = test_case.expected_matches;
        let actual_matches = matches.len();

        if actual_matches != expected_matches {
            trace!("Test case '{}' failed", test_case.name);
            panic!("DEBUGGING");
            return Err(format!(
                "Test case '{}' failed: expected {} matches, got {}",
                test_case.name, expected_matches, actual_matches
            )
            .into());
        }
        Ok(())
    }
}
