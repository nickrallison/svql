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
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndGate"),
    };
    static ref AND_NOR: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/and/verilog/and_nor.v",
        module: "and_nor",
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndNor"),
    };
    static ref AND_MUX: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/and/verilog/and_mux.v",
        module: "and_mux",
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndMux"),
    };
    static ref SDFFE: Pattern = Pattern::Netlist {
        path: "examples/patterns/basic/ff/verilog/sdffe.v",
        module: "sdffe",
        pattern_query_type: Some("svql_query::queries::netlist::basic::dff::Sdffe"),
    };
}

// SECURITY: locked_reg (RTLIL patterns)
lazy_static::lazy_static! {
    static ref ASYNC_EN_IL: Pattern = Pattern::Netlist {
        path: "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
        module: "async_en",
        pattern_query_type: None,
    };
    static ref ASYNC_MUX_IL: Pattern = Pattern::Netlist {
        path: "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
        module: "async_mux",
        pattern_query_type: None,
    };
    static ref SYNC_EN_IL: Pattern = Pattern::Netlist {
        path: "examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il",
        module: "sync_en",
        pattern_query_type: None,
    };
    static ref SYNC_MUX_IL: Pattern = Pattern::Netlist {
        path: "examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il",
        module: "sync_mux",
        pattern_query_type: None,
    };
}

// #####################
// COMPOSITE NEEDLES
// #####################
lazy_static::lazy_static! {
    static ref SDFFE_THEN_AND: Pattern = Pattern::Composite {
        pattern_query_type: "svql_query::queries::netlist::composite::SdffeThenAnd",
    };
    static ref AND_ANY: Pattern = Pattern::Composite {
        pattern_query_type: "svql_query::queries::netlist::composite::AndAny",
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
    static ref SEQ_2_WIDTH_2_SDFFE: Haystack = Haystack {
        path: "examples/fixtures/basic/ff/verilog/seq_2_width_2_sdffe.v",
        module: "seq_2_width_2_sdffe",
    };
    static ref SEQ_1_WIDTH_2_SDFFE: Haystack = Haystack {
        path: "examples/fixtures/basic/ff/verilog/seq_1_width_2_sdffe.v",
        module: "seq_1_width_2_sdffe",
    };
    // Mixed tree provided as a yosys JSON (as per your current layout)
    static ref MIXED_AND_TREE: Haystack = Haystack {
        path: "examples/fixtures/basic/and/json/mixed_and_tree.json",
        module: "mixed_and_tree",
    };
    // New haystacks for the pure-submodule trees
    static ref AND_NOR_TREE: Haystack = Haystack {
        path: "examples/fixtures/basic/and/verilog/and_nor_tree.v",
        module: "and_nor_tree",
    };
    static ref AND_MUX_TREE: Haystack = Haystack {
        path: "examples/fixtures/basic/and/verilog/and_mux_tree.v",
        module: "and_mux_tree",
    };
    // Self haystacks for direct self-matching
    static ref AND_NOR_SELF: Haystack = Haystack {
        path: "examples/patterns/basic/and/verilog/and_nor.v",
        module: "and_nor",
    };
    static ref AND_MUX_SELF: Haystack = Haystack {
        path: "examples/patterns/basic/and/verilog/and_mux.v",
        module: "and_mux",
    };
    // New haystacks for constant checking on and_mux
    static ref AND_MUX_CONST_VARIANTS: Haystack = Haystack {
        path: "examples/fixtures/basic/and/verilog/and_mux_const_variants.v",
        module: "and_mux_const_variants",
    };
    static ref AND_MUX_CONST_BAD: Haystack = Haystack {
        path: "examples/fixtures/basic/and/verilog/and_mux_const_bad.v",
        module: "and_mux_const_bad",
    };
}

// SECURITY: locked_reg (RTLIL haystack)
lazy_static::lazy_static! {
    static ref MANY_LOCKED_REGS_IL: Haystack = Haystack {
        path: "examples/fixtures/security/access_control/locked_reg/rtlil/many_locked_regs.il",
        module: "many_locked_regs",
    };
}

// #####################
// TEST CASES
// #####################
lazy_static::lazy_static! {
    pub static ref BASIC_TEST_CASES: Vec<TestCase> = vec![
        // AND gate tests
        TestCase {
            name: "and_gate_self_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &AND_GATE,
            haystack: &AND_GATE_SELF,
            expected_matches: 2,
        },
        TestCase {
            name: "and_gate_self_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_GATE,
            haystack: &AND_GATE_SELF,
            expected_matches: 1,
        },
        TestCase {
            name: "and_gate_in_and_q_double_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &AND_GATE,
            haystack: &AND_Q_DOUBLE_SDFFE,
            expected_matches: 4,
        },
        TestCase {
            name: "and_gate_in_and_q_double_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_GATE,
            haystack: &AND_Q_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "and_gate_in_and_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_GATE,
            haystack: &AND_TREE,
            expected_matches: 7,
        },
        TestCase {
            name: "and_gate_in_and_seq_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_GATE,
            haystack: &AND_SEQ,
            expected_matches: 7,
        },

        // DFF tests
        TestCase {
            name: "sdffe_self_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &SDFFE_SELF,
            expected_matches: 1,
        },
        TestCase {
            name: "sdffe_in_comb_d_double_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &COMB_D_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_and_q_double_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &AND_Q_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_par_double_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &PAR_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_seq_double_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &SEQ_DOUBLE_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_seq_1_width_2_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &SEQ_1_WIDTH_2_SDFFE,
            expected_matches: 2,
        },
        TestCase {
            name: "sdffe_in_seq_2_width_2_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE,
            haystack: &SEQ_2_WIDTH_2_SDFFE,
            expected_matches: 4,
        },

        // =========================
        // and_nor — positive cases
        // =========================
        TestCase {
            name: "and_nor_self_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_NOR,
            haystack: &AND_NOR_SELF,
            expected_matches: 1,
        },
        TestCase {
            name: "and_nor_in_and_nor_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_NOR,
            haystack: &AND_NOR_TREE,
            expected_matches: 7, // 7 submodule instances
        },
        TestCase {
            name: "and_nor_in_mixed_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_NOR,
            haystack: &MIXED_AND_TREE,
            expected_matches: 2, // two and_nor instances in mixed tree
        },

        // =========================
        // and_nor — negative cases
        // =========================
        TestCase {
            name: "and_nor_in_and_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_NOR,
            haystack: &AND_TREE,
            expected_matches: 0,
        },

        // =========================
        // and_mux — positive cases
        // =========================
        TestCase {
            name: "and_mux_self_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &AND_MUX_SELF,
            expected_matches: 1,
        },
        TestCase {
            name: "and_mux_in_and_mux_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &AND_MUX_TREE,
            expected_matches: 7, // 7 submodule instances
        },
        TestCase {
            name: "and_mux_in_mixed_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &MIXED_AND_TREE,
            expected_matches: 2, // two and_mux instances in mixed tree
        },
        // Constant correctness: one correct + two incorrect in same haystack
        TestCase {
            name: "and_mux_const_variants_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &AND_MUX_CONST_VARIANTS,
            expected_matches: 1, // only the arm with 1'b0 on false branch must match
        },

        // =========================
        // and_mux — negative cases
        // =========================
        TestCase {
            name: "and_mux_in_and_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &AND_TREE,
            expected_matches: 0,
        },
        // Constant correctness: only incorrect variants present
        TestCase {
            name: "and_mux_const_bad_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &AND_MUX_CONST_BAD,
            expected_matches: 0,
        },
    ];

    pub static ref DEDUPE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_nor_dedupe_none_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &AND_NOR,
            haystack: &MIXED_AND_TREE,
            expected_matches: 4,
        },
        TestCase {
            name: "and_nor_dedupe_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_NOR,
            haystack: &MIXED_AND_TREE,
            expected_matches: 2,
        },
        TestCase {
            name: "and_mux_dedupe_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_MUX,
            haystack: &MIXED_AND_TREE,
            expected_matches: 2,
        },
    ];

        pub static ref SECURITY_TEST_CASES: Vec<TestCase> = vec![
        // From many_locked_regs.v there are exactly 2 instances of each:
        // 2 × async_en, 2 × async_mux, 2 × sync_en, 2 × sync_mux

        TestCase {
            name: "async_en_in_many_locked_regs_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &ASYNC_EN_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
        TestCase {
            name: "async_mux_in_many_locked_regs_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &ASYNC_MUX_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
        TestCase {
            name: "sync_en_in_many_locked_regs_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &SYNC_EN_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
        TestCase {
            name: "sync_mux_in_many_locked_regs_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &SYNC_MUX_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
    ];


    pub static ref COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "sdffe_then_and_simple_none",
            config: Config::builder().exact_length().none().flatten().build(),
            pattern: &SDFFE_THEN_AND,
            haystack: &AND_Q_DOUBLE_SDFFE,
            expected_matches: 4,
        },
    ];

    pub static ref ENUM_COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_any_in_mixed_tree_auto_morph",
            config: Config::builder().exact_length().auto_morph().flatten().build(),
            pattern: &AND_ANY,
            haystack: &MIXED_AND_TREE,
            expected_matches: 7,  // 3 gates + 2 muxes + 2 nors
        },
    ];

    pub static ref ALL_TEST_CASES: Vec<TestCase> = {
        let mut all = BASIC_TEST_CASES.clone();
        all.extend(DEDUPE_TEST_CASES.clone());
        all.extend(SECURITY_TEST_CASES.clone());
        all.extend(COMPOSITE_TEST_CASES.clone());
        all.extend(ENUM_COMPOSITE_TEST_CASES.clone());
        all
    };
}
