#[cfg(test)]
mod tests {
    use crate::SubgraphMatcher;
    use svql_common::{BASIC_TEST_CASES, Needle};

    #[test]
    fn test_basic_subgraph_matches() {
        for case in BASIC_TEST_CASES.iter() {
            // We only port Netlist-based tests to the subgraph kernel
            let Needle::Netlist { yosys_module, .. } = case.needle else {
                continue;
            };

            // 1. Import designs using the common Yosys wrapper
            let needle_design = yosys_module
                .import_design(&case.config.needle_options)
                .expect(&format!("Failed to import needle for {}", case.name));

            let haystack_design = case
                .haystack
                .yosys_module
                .import_design(&case.config.haystack_options)
                .expect(&format!("Failed to import haystack for {}", case.name));

            // 2. Execute the subgraph matcher
            let assignments = SubgraphMatcher::enumerate_all(
                &needle_design,
                &haystack_design,
                yosys_module.module_name().to_string(),
                case.haystack.yosys_module.module_name().to_string(),
                &case.config,
            );

            // 3. Validate match count
            assert_eq!(
                assignments.len(),
                case.expected_matches,
                "Test '{}' failed: expected {} matches, found {}",
                case.name,
                case.expected_matches,
                assignments.len()
            );
        }
    }
}
