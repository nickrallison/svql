#[cfg(test)]
mod tests {
    use svql_common::{ALL_TEST_CASES, Pattern};
    use svql_driver::Driver;
    use svql_query::Search;

    // Include the generated registry
    include!(concat!(env!("OUT_DIR"), "/svql_query_registry.rs"));

    #[test]
    fn test_all_query_cases() {
        env_logger::init();

        let driver = Driver::new_workspace().expect("Failed to create driver");

        let test_cases = ALL_TEST_CASES.iter().filter(|tc| {
            matches!(
                tc.pattern,
                Pattern::Netlist {
                    pattern_query_type: Some(_),
                    ..
                }
            ) || matches!(tc.pattern, Pattern::Composite { .. })
        });

        let test_results: Vec<Result<(), Box<dyn std::error::Error>>> = test_cases
            .map(|test_case| run_query_test_case(&driver, test_case))
            .collect();

        let num_errors = test_results.iter().filter(|r| r.is_err()).count();

        if num_errors > 0 {
            eprintln!(
                "{} out of {} query tests failed",
                num_errors,
                test_results.len()
            );
            for result in test_results {
                match result {
                    Ok(()) => {}
                    Err(e) => eprintln!("Query test failed: {}", e),
                }
            }
            assert!(false, "Some query test cases failed");
        }
    }

    fn run_query_test_case(
        driver: &Driver,
        test_case: &svql_common::TestCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query_type_name = match test_case.pattern {
            Pattern::Netlist {
                pattern_query_type: Some(name),
                ..
            } => name,
            Pattern::Composite {
                pattern_query_type: name,
            } => name,
            _ => return Ok(()), // Skip test cases without query types
        };

        // Run the query using the generated registry
        let result = run_count_by_name(query_type_name, driver, test_case)
            .ok_or_else(|| format!("Unknown query type: {}", query_type_name))?;

        match result {
            Ok(actual_matches) => {
                let expected_matches = test_case.expected_matches;

                if actual_matches != expected_matches {
                    return Err(format!(
                        "Query test case '{}' ({}) failed: expected {} matches, got {}",
                        test_case.name, query_type_name, expected_matches, actual_matches
                    )
                    .into());
                }
                Ok(())
            }
            Err(e) => Err(format!("Query test case '{}' failed: {}", test_case.name, e).into()),
        }
    }
}
