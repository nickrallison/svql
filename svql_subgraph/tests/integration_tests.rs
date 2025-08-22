#[cfg(test)]
mod tests {
    use svql_common::{ALL_TEST_CASES, Pattern};
    use svql_subgraph::test_support::load_design_from;

    #[test]
    fn test_all_netlist_cases() {
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
        let (netlist_pattern_path, _netlist_pattern_module) = match test_case.pattern {
            Pattern::Netlist { path, module, .. } => (path, module),
            _ => panic!("Test case is not a netlist pattern"),
        };

        let needle = load_design_from(netlist_pattern_path)
            .expect(&format!("Failed to load needle: {}", netlist_pattern_path));
        let haystack = load_design_from(test_case.haystack.path).expect(&format!(
            "Failed to load haystack: {}",
            test_case.haystack.path
        ));

        let matches = svql_subgraph::find_subgraphs(&needle, &haystack, &test_case.config);

        let expected_matches = test_case.expected_matches;
        let actual_matches = matches.len();

        if actual_matches != expected_matches {
            return Err(format!(
                "Test case '{}' failed: expected {} matches, got {}",
                test_case.name, expected_matches, actual_matches
            )
            .into());
        }
        Ok(())
    }
}
