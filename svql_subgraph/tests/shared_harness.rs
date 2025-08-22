#[cfg(test)]
mod tests {
    use svql_common::{
        ALL_TEST_CASES, BASIC_TEST_CASES, COMPOSITE_TEST_CASES, DEDUPE_TEST_CASES,
        ENUM_COMPOSITE_TEST_CASES,
    };
    use svql_subgraph::test_support::load_design_from;

    #[test]
    fn test_all_basic_cases() {
        for test_case in BASIC_TEST_CASES.iter() {
            run_test_case(test_case);
        }
    }

    #[test]
    fn test_all_dedupe_cases() {
        for test_case in DEDUPE_TEST_CASES.iter() {
            run_test_case(test_case);
        }
    }

    #[test]
    fn test_all_composite_cases() {
        for test_case in COMPOSITE_TEST_CASES.iter() {
            run_test_case(test_case);
        }
    }

    #[test]
    fn test_all_enum_composite_cases() {
        for test_case in ENUM_COMPOSITE_TEST_CASES.iter() {
            run_test_case(test_case);
        }
    }

    #[test]
    fn test_all_cases() {
        for test_case in ALL_TEST_CASES.iter() {
            run_test_case(test_case);
        }
    }

    fn run_test_case(test_case: &svql_common::TestCase) {
        let needle = load_design_from(test_case.pattern_path).expect(&format!(
            "Failed to load needle: {}",
            test_case.pattern_path
        ));
        let haystack = load_design_from(test_case.haystack_path).expect(&format!(
            "Failed to load haystack: {}",
            test_case.haystack_path
        ));

        let matches = svql_subgraph::find_subgraphs(&needle, &haystack, &test_case.config);

        assert_eq!(
            matches.len(),
            test_case.expected_matches,
            "Test case '{}' failed: expected {} matches, got {}",
            test_case.name,
            test_case.expected_matches,
            matches.len()
        );
    }
}
