use rstest::rstest;
use svql_subgraph::test_support::load_design_from;
use svql_test_data::TestCase;

#[cfg(test)]
mod tests {

    #[rstest]
    #[case::and_gate_self(
        "examples/patterns/basic/and/verilog/and_gate.v",
        "examples/patterns/basic/and/verilog/and_gate.v",
        1
    )]
    #[case::and_gate_in_and_q_double(
        "examples/patterns/basic/and/verilog/and_gate.v",
        "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
        2
    )]
    #[case::and_gate_in_and_tree(
        "examples/patterns/basic/and/verilog/and_gate.v",
        "examples/fixtures/basic/and/verilog/and_tree.v",
        7
    )]
    #[case::and_gate_in_and_seq(
        "examples/patterns/basic/and/verilog/and_gate.v",
        "examples/fixtures/basic/and/verilog/and_seq.v",
        7
    )]
    #[case::sdffe_self(
        "examples/patterns/basic/ff/verilog/sdffe.v",
        "examples/patterns/basic/ff/verilog/sdffe.v",
        1
    )]
    #[case::sdffe_in_comb_d_double(
        "examples/patterns/basic/ff/verilog/sdffe.v",
        "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v",
        2
    )]
    #[case::sdffe_in_and_q_double(
        "examples/patterns/basic/ff/verilog/sdffe.v",
        "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
        2
    )]
    #[case::sdffe_in_par_double(
        "examples/patterns/basic/ff/verilog/sdffe.v",
        "examples/fixtures/basic/ff/verilog/par_double_sdffe.v",
        2
    )]
    #[case::sdffe_in_seq_double(
        "examples/patterns/basic/ff/verilog/sdffe.v",
        "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
        2
    )]
    fn test_subgraph_matches(
        #[case] needle_path: &str,
        #[case] haystack_path: &str,
        #[case] expected: usize,
    ) {
        let needle = load_design_from(needle_path)
            .expect(&format!("Failed to load needle: {}", needle_path));
        let haystack = load_design_from(haystack_path)
            .expect(&format!("Failed to load haystack: {}", haystack_path));
        let config = svql_subgraph::Config::builder()
            .exact_length()
            .none()
            .build();

        let matches = svql_subgraph::find_subgraphs(&needle, &haystack, &config);

        assert_eq!(
            matches.len(),
            expected,
            "Expected {} matches for needle {}, against haystack {}, got {}",
            expected,
            needle_path,
            haystack_path,
            matches.len()
        );
    }

    #[rstest]
    #[case::dedupe_none(&svql_test_data::DEDUPE_TEST_CASES[0])]
    #[case::dedupe_auto_morph(&svql_test_data::DEDUPE_TEST_CASES[1])]
    #[case::dedupe_auto_morph_and_mux(&svql_test_data::DEDUPE_TEST_CASES[2])]
    fn test_dedupe_modes(#[case] test_case: &TestCase) {
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
