use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Pattern {
    Netlist {
        path: &'static str,
        module: &'static str,
        pattern_query_type: Option<&'static str>,
    },
    Composite {
        pattern_query_type: &'static str,
    },
}

impl Pattern {
    pub fn is_netlist(&self) -> bool {
        matches!(self, Pattern::Netlist { .. })
    }

    pub fn path(&self) -> &'static str {
        match self {
            Pattern::Netlist { path, .. } => path,
            Pattern::Composite { .. } => panic!("Composite patterns don't have paths"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Haystack {
    pub path: &'static str,
    pub module: &'static str,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: &'static str,
    pub config: Config,
    pub pattern: &'static Pattern,
    pub haystack: &'static Haystack,
    pub expected_matches: usize,
}

// #####################
// NEEDLES (Patterns)
// #####################
lazy_static::lazy_static! {
    static ref AND_GATE: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/and/verilog/and_gate.v",
        module: "and_gate",
        pattern_query_type: Some("AndGate"),
    };
    static ref AND_NOR: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/and/verilog/and_nor.v",
        module: "and_nor",
        pattern_query_type: Some("AndNor"),
    };
    static ref AND_MUX: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/and/verilog/and_mux.v",
        module: "and_mux",
        pattern_query_type: Some("AndMux"),
    };
    static ref SDFFE: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/ff/verilog/sdffe.v",
        module: "sdffe",
        pattern_query_type: Some("Sdffe"),
    };
}

// #####################
// HAYSTACKS (Targets)
// #####################
lazy_static::lazy_static! {
    static ref AND_GATE_SELF: Haystack = Haystack {
        path: "examples/patterns/basic/and/verilog/and_gate.v",
        module: "and_gate",
    };
    static ref AND_Q_DOUBLE_SDFFE: Haystack = Haystack {
        path: "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
        module: "and_q_double_sdffe",
    };
    static ref AND_TREE: Haystack = Haystack {
        path: "examples/fixtures/basic/and/verilog/and_tree.v",
        module: "and_tree",
    };
    static ref AND_SEQ: Haystack = Haystack {
        path: "examples/fixtures/basic/and/verilog/and_seq.v",
        module: "and_seq",
    };
    static ref SDFFE_SELF: Haystack = Haystack {
        path: "examples/patterns/basic/ff/verilog/sdffe.v",
        module: "sdffe",
    };
    static ref COMB_D_DOUBLE_SDFFE: Haystack = Haystack {
        path: "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v",
        module: "comb_d_double_sdffe",
    };
    static ref PAR_DOUBLE_SDFFE: Haystack = Haystack {
        path: "examples/fixtures/basic/ff/verilog/par_double_sdffe.v",
        module: "par_double_sdffe",
    };
    static ref SEQ_DOUBLE_SDFFE: Haystack = Haystack {
        path: "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
        module: "seq_double_sdffe",
    };
    static ref MIXED_AND_TREE: Haystack = Haystack {
        path: "examples/fixtures/basic/and/json/mixed_and_tree.json",
        module: "mixed_and_tree",
    };
}

// #####################
// TEST CASES
// #####################
lazy_static::lazy_static! {
    pub static ref BASIC_TEST_CASES: Vec<TestCase> = vec![
        // AND gate tests
        TestCase {
            name: "and_gate_self",
            config: Config::builder().exact_length().none().build(),
            pattern: &*AND_GATE,
            haystack: &*AND_GATE_SELF,
            expected_matches: 1,
        },
        TestCase {
            name: "and_gate_in_and_q_double",
            config: Config::builder().exact_length().none().build(),
            pattern: &*AND_GATE,
            haystack: &*AND_Q_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "and_gate_in_and_tree",
            config: Config::builder().exact_length().none().build(),
            pattern: &*AND_GATE,
            haystack: &*AND_TREE,
            expected_matches: 7,
        },
        TestCase {
            name: "and_gate_in_and_seq",
            config: Config::builder().exact_length().none().build(),
            pattern: &*AND_GATE,
            haystack: &*AND_SEQ,
            expected_matches: 7,
        },

        // DFF tests
        TestCase {
            name: "sdffe_self",
            config: Config::builder().exact_length().none().build(),
            pattern: &*SDFFE,
            haystack: &*SDFFE_SELF,
            expected_matches: 1,
        },
        TestCase {
            name: "sdffe_in_comb_d_double",
            config: Config::builder().exact_length().none().build(),
            pattern: &*SDFFE,
            haystack: &*COMB_D_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_and_q_double",
            config: Config::builder().exact_length().none().build(),
            pattern: &*SDFFE,
            haystack: &*AND_Q_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_par_double",
            config: Config::builder().exact_length().none().build(),
            pattern: &*SDFFE,
            haystack: &*PAR_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_seq_double",
            config: Config::builder().exact_length().none().build(),
            pattern: &*SDFFE,
            haystack: &*SEQ_DOUBLE_SDFFE,
            expected_matches: 2,
        },
    ];

    pub static ref DEDUPE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_nor_dedupe_none",
            config: Config::builder().exact_length().none().build(),
            pattern: &*AND_NOR,
            haystack: &*MIXED_AND_TREE,
            expected_matches: 4,
        },
        TestCase {
            name: "and_nor_dedupe_auto_morph",
            config: Config::builder().exact_length().auto_morph().build(),
            pattern: &*AND_NOR,
            haystack: &*MIXED_AND_TREE,
            expected_matches: 2,
        },
        TestCase {
            name: "and_mux_dedupe_auto_morph",
            config: Config::builder().exact_length().auto_morph().build(),
            pattern: &*AND_MUX,
            haystack: &*MIXED_AND_TREE,
            expected_matches: 2,
        },
    ];

    pub static ref COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "sdffe_then_and_simple",
            config: Config::builder().exact_length().none().build(),
            pattern: &Pattern::Composite {
                pattern_query_type: "sdffe_then_and",
            },
            haystack: &*AND_Q_DOUBLE_SDFFE,
            expected_matches: 2,
        },
    ];

    pub static ref ENUM_COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_any_in_mixed_tree",
            config: Config::builder().exact_length().auto_morph().build(),
            pattern: &Pattern::Composite {
                pattern_query_type: "and_any",
            },
            haystack: &*MIXED_AND_TREE,
            expected_matches: 7,  // 3 gates + 2 muxes + 2 nors
        },
    ];

    pub static ref ALL_TEST_CASES: Vec<TestCase> = {
        let mut all = BASIC_TEST_CASES.clone();
        all.extend(DEDUPE_TEST_CASES.clone());
        all.extend(COMPOSITE_TEST_CASES.clone());
        all.extend(ENUM_COMPOSITE_TEST_CASES.clone());
        all
    };
}
