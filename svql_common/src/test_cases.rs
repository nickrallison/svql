use std::path::Path;

use crate::{YosysModule, config::Config};

#[derive(Debug, Clone)]
pub enum Pattern {
    Netlist {
        yosys_module: YosysModule,
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

    pub fn path(&self) -> &Path {
        match self {
            Pattern::Netlist { yosys_module, .. } => yosys_module.path(),
            Pattern::Composite { .. } => panic!("Composite patterns don't have paths"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Haystack {
    pub yosys_module: YosysModule,
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
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "and_gate",
        ).expect("Failed to create YosysModule for and_gate"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndGate"),
    };
    static ref AND_NOR: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_nor.v",
            "and_nor",
        ).expect("Failed to create YosysModule for and_nor"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndNor"),
    };
    static ref AND_MUX: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_mux.v",
            "and_mux",
        ).expect("Failed to create YosysModule for and_mux"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndMux"),
    };
    static ref SDFFE: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/ff/rtlil/sdffe.il",
            "sdffe",
        ).expect("Failed to create YosysModule for sdffe"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::dff::Sdffe"),
    };
    static ref AND_SEQ: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "and_seq",
        ).expect("Failed to create YosysModule for and_seq"),
        pattern_query_type: None,
    };
    static ref SMALL_AND_SEQ: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/small_and_seq.v",
            "small_and_seq",
        ).expect("Failed to create YosysModule for small_and_seq"),
        pattern_query_type: None,
    };
}

// SECURITY: locked_reg (RTLIL patterns)
lazy_static::lazy_static! {
    static ref ASYNC_EN_IL: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
            "async_en",
        ).expect("Failed to create YosysModule for async_en"),
        pattern_query_type: None,
    };
    static ref ASYNC_MUX_IL: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
            "async_mux",
        ).expect("Failed to create YosysModule for async_mux"),
        pattern_query_type: None,
    };
    static ref SYNC_EN_IL: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il",
            "sync_en",
        ).expect("Failed to create YosysModule for sync_en"),
        pattern_query_type: None,
    };
    static ref SYNC_MUX_IL: Pattern = Pattern::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il",
            "sync_mux",
        ).expect("Failed to create YosysModule for sync_mux"),
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
    static ref AND_GATE_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "and_gate",
        ).expect("Failed to create YosysModule for and_gate"),
    };
    static ref AND_Q_DOUBLE_SDFFE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "and_q_double_sdffe",
        ).expect("Failed to create YosysModule for and_q_double_sdffe"),
    };
    static ref AND_TREE_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_tree.v",
            "and_tree",
        ).expect("Failed to create YosysModule for and_tree"),
    };
    static ref SMALL_AND_TREE_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/small_and_tree.v",
            "small_and_tree",
        ).expect("Failed to create YosysModule for small_and_tree"),
    };
    static ref AND_SEQ_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "and_seq",
        ).expect("Failed to create YosysModule for and_seq"),
    };
    static ref SDFFE_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/ff/rtlil/sdffe.il",
            "sdffe",
        ).expect("Failed to create YosysModule for sdffe"),
    };
    static ref COMB_D_DOUBLE_SDFFE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v",
            "comb_d_double_sdffe",
        ).expect("Failed to create YosysModule for comb_d_double_sdffe"),
    };
    static ref PAR_DOUBLE_SDFFE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/par_double_sdffe.v",
            "par_double_sdffe",
        ).expect("Failed to create YosysModule for par_double_sdffe"),
    };
    static ref SEQ_DOUBLE_SDFFE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
            "seq_double_sdffe",
        ).expect("Failed to create YosysModule for seq_double_sdffe"),
    };
    static ref SEQ_2_WIDTH_2_SDFFE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/rtlil/seq_2_width_2_sdffe.il",
            "seq_2_width_2_sdffe",
        ).expect("Failed to create YosysModule for seq_2_width_2_sdffe"),
    };
    static ref SEQ_1_WIDTH_2_SDFFE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/rtlil/seq_1_width_2_sdffe.il",
            "seq_1_width_2_sdffe",
        ).expect("Failed to create YosysModule for seq_1_width_2_sdffe"),
    };
    // Mixed tree provided as a yosys JSON (as per your current layout)
    static ref MIXED_AND_TREE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/json/mixed_and_tree.json",
            "mixed_and_tree",
        ).expect("Failed to create YosysModule for mixed_and_tree"),
    };
    // New haystacks for the pure-submodule trees
    static ref AND_NOR_TREE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_nor_tree.v",
            "and_nor_tree",
        ).expect("Failed to create YosysModule for and_nor_tree"),
    };
    static ref AND_MUX_TREE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_mux_tree.v",
            "and_mux_tree",
        ).expect("Failed to create YosysModule for and_mux_tree"),
    };
    // Self haystacks for direct self-matching
    static ref AND_NOR_SELF: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_nor.v",
            "and_nor",
        ).expect("Failed to create YosysModule for and_nor"),
    };
    static ref AND_MUX_SELF: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_mux.v",
            "and_mux",
        ).expect("Failed to create YosysModule for and_mux"),
    };
    // New haystacks for constant checking on and_mux
    static ref AND_MUX_CONST_VARIANTS: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_mux_const_variants.v",
            "and_mux_const_variants",
        ).expect("Failed to create YosysModule for and_mux_const_variants"),
    };
    static ref AND_MUX_CONST_BAD: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_mux_const_bad.v",
            "and_mux_const_bad",
        ).expect("Failed to create YosysModule for and_mux_const_bad"),
    };
    static ref SMALL_AND_TREE: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/small_and_tree.v",
            "small_and_tree",
        ).expect("Failed to create YosysModule for small_and_tree"),
    };
}

// SECURITY: locked_reg (RTLIL haystack)
lazy_static::lazy_static! {
    static ref MANY_LOCKED_REGS_IL: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/security/access_control/locked_reg/rtlil/many_locked_regs.il",
            "many_locked_regs",
        ).expect("Failed to create YosysModule for many_locked_regs"),
    };
}

// #####################
// TEST CASES
// #####################
lazy_static::lazy_static! {
    pub static ref BASIC_TEST_CASES: Vec<TestCase> = vec![
        // AND gate tests
        TestCase {
            name: "and_gate_self_dedupe_no",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                .haystack_flatten(true)
                .dedupe(crate::Dedupe::None)
                .build(),
            pattern: &AND_GATE,
            haystack: &AND_GATE_HAYSTACK,
            expected_matches: 2,
        },
        TestCase {
            name: "and_gate_self_dedupe_yes",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                .haystack_flatten(true)
                .dedupe(crate::Dedupe::None)
                .build(),
            pattern: &AND_GATE,
            haystack: &AND_GATE_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "small_and_seq_3_and_tree_4_dedupe_no",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                .haystack_flatten(true)
                .dedupe(crate::Dedupe::None)
                .build(),
            pattern: &SMALL_AND_SEQ,
            haystack: &SMALL_AND_TREE_HAYSTACK,
            expected_matches: 4,
        },
        TestCase {
            name: "small_and_seq_3_and_tree_4_dedupe_yes",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                .haystack_flatten(true)
                .dedupe(crate::Dedupe::None)
                .build(),
            pattern: &SMALL_AND_SEQ,
            haystack: &SMALL_AND_TREE_HAYSTACK,
            expected_matches: 2,
        },

    ];

    pub static ref DEDUPE_TEST_CASES: Vec<TestCase> = vec![
    //     TestCase {
    //         name: "and_nor_dedupe_none_none",
    //         config: Config::builder().exact_length().none().flatten().build(),
    //         pattern: &AND_NOR,
    //         haystack: &MIXED_AND_TREE,
    //         expected_matches: 4,
    //     },
    //     TestCase {
    //         name: "and_nor_dedupe_auto_morph",
    //         config: Config::builder().exact_length().auto_morph().flatten().build(),
    //         pattern: &AND_NOR,
    //         haystack: &MIXED_AND_TREE,
    //         expected_matches: 2,
    //     },
    //     TestCase {
    //         name: "and_mux_dedupe_auto_morph",
    //         config: Config::builder().exact_length().auto_morph().flatten().build(),
    //         pattern: &AND_MUX,
    //         haystack: &MIXED_AND_TREE,
    //         expected_matches: 2,
    //     },
    ];

        pub static ref SECURITY_TEST_CASES: Vec<TestCase> = vec![
        // From many_locked_regs.v there are exactly 2 instances of each:
        // 2 × async_en, 2 × async_mux, 2 × sync_en, 2 × sync_mux

        TestCase {
            name: "async_en_in_many_locked_regs",
            config: Config::builder().match_length(crate::MatchLength::Exact).haystack_flatten(true).build(),
            pattern: &ASYNC_EN_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
        TestCase {
            name: "async_mux_in_many_locked_regs",
            config: Config::builder().match_length(crate::MatchLength::Exact).haystack_flatten(true).build(),
            pattern: &ASYNC_MUX_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
        TestCase {
            name: "sync_en_in_many_locked_regs",
            config: Config::builder().match_length(crate::MatchLength::Exact).haystack_flatten(true).build(),
            pattern: &SYNC_EN_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
        TestCase {
            name: "sync_mux_in_many_locked_regs",
            config: Config::builder().match_length(crate::MatchLength::Exact).haystack_flatten(true).build(),
            pattern: &SYNC_MUX_IL,
            haystack: &MANY_LOCKED_REGS_IL,
            expected_matches: 2,
        },
    ];


    pub static ref COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "sdffe_then_and_simple_none",
            config: Config::builder().match_length(crate::MatchLength::Exact).haystack_flatten(true).build(),
            pattern: &SDFFE_THEN_AND,
            haystack: &AND_Q_DOUBLE_SDFFE,
            expected_matches: 4,
        },
    ];

    pub static ref ENUM_COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_any_in_mixed_tree_auto_morph",
            config: Config::builder().match_length(crate::MatchLength::Exact).haystack_flatten(true).build(),
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
