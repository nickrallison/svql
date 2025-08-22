use svql_subgraph::Config;

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: &'static str,
    pub pattern_path: &'static str,
    pub pattern_module: &'static str,
    pub haystack_path: &'static str,
    pub haystack_module: &'static str,
    pub expected_matches: usize,
    pub config: Config,
}

lazy_static::lazy_static! {
    pub static ref BASIC_TEST_CASES: Vec<TestCase> = vec![
        // AND gate tests
        TestCase {
            name: "and_gate_self",
            pattern_path: "examples/patterns/basic/and/verilog/and_gate.v",
            pattern_module: "and_gate",
            haystack_path: "examples/patterns/basic/and/verilog/and_gate.v",
            haystack_module: "and_gate",
            expected_matches: 1,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "and_gate_in_and_q_double",
            pattern_path: "examples/patterns/basic/and/verilog/and_gate.v",
            pattern_module: "and_gate",
            haystack_path: "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            haystack_module: "and_q_double_sdffe",
            expected_matches: 2,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "and_gate_in_and_tree",
            pattern_path: "examples/patterns/basic/and/verilog/and_gate.v",
            pattern_module: "and_gate",
            haystack_path: "examples/fixtures/basic/and/verilog/and_tree.v",
            haystack_module: "and_tree",
            expected_matches: 7,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "and_gate_in_and_seq",
            pattern_path: "examples/patterns/basic/and/verilog/and_gate.v",
            pattern_module: "and_gate",
            haystack_path: "examples/fixtures/basic/and/verilog/and_seq.v",
            haystack_module: "and_seq",
            expected_matches: 7,
            config: Config::builder().exact_length().none().build(),
        },

        // DFF tests
        TestCase {
            name: "sdffe_self",
            pattern_path: "examples/patterns/basic/ff/verilog/sdffe.v",
            pattern_module: "sdffe",
            haystack_path: "examples/patterns/basic/ff/verilog/sdffe.v",
            haystack_module: "sdffe",
            expected_matches: 1,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "sdffe_in_comb_d_double",
            pattern_path: "examples/patterns/basic/ff/verilog/sdffe.v",
            pattern_module: "sdffe",
            haystack_path: "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v",
            haystack_module: "comb_d_double_sdffe",
            expected_matches: 2,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "sdffe_in_and_q_double",
            pattern_path: "examples/patterns/basic/ff/verilog/sdffe.v",
            pattern_module: "sdffe",
            haystack_path: "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            haystack_module: "and_q_double_sdffe",
            expected_matches: 2,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "sdffe_in_par_double",
            pattern_path: "examples/patterns/basic/ff/verilog/sdffe.v",
            pattern_module: "sdffe",
            haystack_path: "examples/fixtures/basic/ff/verilog/par_double_sdffe.v",
            haystack_module: "par_double_sdffe",
            expected_matches: 2,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "sdffe_in_seq_double",
            pattern_path: "examples/patterns/basic/ff/verilog/sdffe.v",
            pattern_module: "sdffe",
            haystack_path: "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
            haystack_module: "seq_double_sdffe",
            expected_matches: 2,
            config: Config::builder().exact_length().none().build(),
        },
    ];

    pub static ref DEDUPE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_nor_dedupe_none",
            pattern_path: "examples/patterns/basic/and/verilog/and_nor.v",
            pattern_module: "and_nor",
            haystack_path: "examples/fixtures/basic/and/json/mixed_and_tree.json",
            haystack_module: "mixed_and_tree",
            expected_matches: 4,
            config: Config::builder().exact_length().none().build(),
        },
        TestCase {
            name: "and_nor_dedupe_auto_morph",
            pattern_path: "examples/patterns/basic/and/verilog/and_nor.v",
            pattern_module: "and_nor",
            haystack_path: "examples/fixtures/basic/and/json/mixed_and_tree.json",
            haystack_module: "mixed_and_tree",
            expected_matches: 2,
            config: Config::builder().exact_length().auto_morph().build(),
        },
        TestCase {
            name: "and_mux_dedupe_auto_morph",
            pattern_path: "examples/patterns/basic/and/verilog/and_mux.v",
            pattern_module: "and_mux",
            haystack_path: "examples/fixtures/basic/and/json/mixed_and_tree.json",
            haystack_module: "mixed_and_tree",
            expected_matches: 2,
            config: Config::builder().exact_length().auto_morph().build(),
        },
    ];

    pub static ref ALL_TEST_CASES: Vec<TestCase> = {
        let mut all = BASIC_TEST_CASES.clone();
        all.extend(DEDUPE_TEST_CASES.clone());
        all
    };
}
