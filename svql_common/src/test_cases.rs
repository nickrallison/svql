//! Common test cases and definitions for SVQL.
//!
//! This module defines the structures for test cases, needles (patterns), and haystacks (targets),
//! and provides a collection of pre-defined test cases.

use std::path::Path;

use crate::{Dedupe, MatchLength, YosysModule, config::Config};

/// Represents a pattern to search for (the "needle").
#[derive(Debug, Clone)]
pub enum Needle {
    /// A netlist pattern loaded from a Yosys module.
    Netlist {
        yosys_module: YosysModule,
        pattern_query_type: Option<&'static str>,
    },
    /// A composite pattern defined in code.
    Composite { pattern_query_type: &'static str },
}

impl Needle {
    pub fn is_netlist(&self) -> bool {
        matches!(self, Needle::Netlist { .. })
    }

    pub fn path(&self) -> &Path {
        match self {
            Needle::Netlist { yosys_module, .. } => yosys_module.path(),
            Needle::Composite { .. } => panic!("Composite patterns don't have paths"),
        }
    }
}

/// Represents a target design to search in (the "haystack").
#[derive(Debug, Clone)]
pub struct Haystack {
    pub yosys_module: YosysModule,
}

/// A complete test case definition.
#[derive(Debug, Clone)]
pub struct TestCase {
    /// The name of the test case.
    pub name: &'static str,
    /// The configuration for the search.
    pub config: Config,
    /// The pattern to search for.
    pub needle: &'static Needle,
    /// The target design to search in.
    pub haystack: &'static Haystack,
    /// The expected number of matches.
    pub expected_matches: usize,
}

// #####################
// NEEDLES (Patterns)
// #####################
lazy_static::lazy_static! {
    static ref AND_GATE: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "and_gate",
        ).expect("Failed to create YosysModule for and_gate"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndGate"),
    };
    static ref OR_GATE: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/or/verilog/or_gate.v",
            "or_gate",
        ).expect("Failed to create YosysModule for or_gate"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::or::OrGate"),
    };
    static ref NOT_GATE: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/not/verilog/not_gate.v",
            "not_gate",
        ).expect("Failed to create YosysModule for not_gate"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::not::NotGate"),
    };

    static ref AND_NOR: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_nor.v",
            "and_nor",
        ).expect("Failed to create YosysModule for and_nor"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndNor"),
    };
    static ref AND_MUX: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_mux.v",
            "and_mux",
        ).expect("Failed to create YosysModule for and_mux"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::and::AndMux"),
    };
    static ref SDFFE: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/ff/rtlil/sdffe.il",
            "sdffe",
        ).expect("Failed to create YosysModule for sdffe"),
        pattern_query_type: Some("svql_query::queries::netlist::basic::dff::Sdffe"),
    };
    static ref AND_SEQ: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "and_seq",
        ).expect("Failed to create YosysModule for and_seq"),
        pattern_query_type: None,
    };
    static ref SMALL_AND_SEQ: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/and/verilog/small_and_seq.v",
            "small_and_seq",
        ).expect("Failed to create YosysModule for small_and_seq"),
        pattern_query_type: None,
    };

    // ----------
    static ref DFF_LOOP_TOGGLE: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/dff_loop_toggle.v",
            "dff_loop_toggle",
        ).expect("Failed to create YosysModule for dff_loop_toggle"),
        pattern_query_type: None,
    };
    static ref DFF_NOT: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/dff_not.v",
            "dff_not",
        ).expect("Failed to create YosysModule for dff_not"),
        pattern_query_type: None,
    };
    static ref NOT_DFF: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/not_dff.v",
            "not_dff",
        ).expect("Failed to create YosysModule for not_dff"),
        pattern_query_type: None,
    };

}

// SECURITY: locked_reg (RTLIL patterns)
lazy_static::lazy_static! {
    static ref ASYNC_EN_IL: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
            "async_en",
        ).expect("Failed to create YosysModule for async_en"),
        pattern_query_type: None,
    };
    static ref ASYNC_MUX_IL: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
            "async_mux",
        ).expect("Failed to create YosysModule for async_mux"),
        pattern_query_type: None,
    };
    static ref SYNC_EN_IL: Needle = Needle::Netlist {
        yosys_module: YosysModule::new(
            "examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il",
            "sync_en",
        ).expect("Failed to create YosysModule for sync_en"),
        pattern_query_type: None,
    };
    static ref SYNC_MUX_IL: Needle = Needle::Netlist {
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
    static ref SDFFE_THEN_AND: Needle = Needle::Composite {
        pattern_query_type: "svql_query::queries::composites::dff_then_and::SdffeThenAnd",
    };
    static ref AND_ANY: Needle = Needle::Composite {
        pattern_query_type: "svql_query::variants::and_any::AndAny",
    };
    static ref REC_AND: Needle = Needle::Composite {
        pattern_query_type: "svql_query::queries::composites::rec_and::RecAnd",
    };
    static ref REC_OR: Needle = Needle::Composite {
        pattern_query_type: "svql_query::queries::composites::rec_or::RecOr",
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
    static ref AND_2_SEQ_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_2_seq.v",
            "and_2_seq",
        ).expect("Failed to create YosysModule for and_2_seq"),
    };
    static ref SINGLE_AND_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "and_gate",
        ).expect("Failed to create YosysModule for single AND"),
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
    static ref OR_2_SEQ_HAYSTACK: Haystack = Haystack {  // NEW: Sequential OR chain (N=3)
        yosys_module: YosysModule::new(
            "examples/patterns/basic/or/verilog/or_2_seq.v",  // NEW: Assume this file exists
            "or_2_seq",
        ).expect("Failed to create YosysModule for or_2_seq"),
    };

    static ref SINGLE_OR_HAYSTACK: Haystack = Haystack {  // NEW: Single OR for base case
        yosys_module: YosysModule::new(
            "examples/patterns/basic/or/verilog/or_gate.v",
            "or_gate",
        ).expect("Failed to create YosysModule for single OR"),
    };
    static ref NOT_GATE_HAYSTACK: Haystack = Haystack {  // NEW: Self haystack for NOT
        yosys_module: YosysModule::new(
            "examples/patterns/basic/not/verilog/not_gate.v",
            "not_gate",
        ).expect("Failed to create YosysModule for not_gate"),
    };
    static ref NOT_2_SEQ_HAYSTACK: Haystack = Haystack {  // NEW: Sequential NOT chain (N=2)
        yosys_module: YosysModule::new(
            "examples/patterns/basic/not/verilog/not_2_seq.v",
            "not_2_seq",
        ).expect("Failed to create YosysModule for not_2_seq"),
    };

    static ref DFF_LOOP_TOGGLE_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/dff_loop_toggle.v",
            "dff_loop_toggle",
        ).expect("Failed to create YosysModule for dff_loop_toggle"),
    };
    static ref DFF_NOT_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/dff_not.v",
            "dff_not",
        ).expect("Failed to create YosysModule for dff_not"),
    };
    static ref NOT_DFF_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/basic/ff/verilog/not_dff.v",
            "not_dff",
        ).expect("Failed to create YosysModule for not_dff"),
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
    static ref CWE1234_HAYSTACK: Haystack = Haystack {
        yosys_module: YosysModule::new(
            "examples/fixtures/security/access_control/locked_reg/verilog/cwe1234.v",
            "cwe1234",
        ).expect("Failed to create YosysModule for cwe1234"),
    };
}

// #####################
// TEST CASES
// #####################
lazy_static::lazy_static! {
    pub static ref BASIC_TEST_CASES: Vec<TestCase> = vec![
        // AND gate tests
        TestCase {
            name: "and_gate_self_dedupe_none",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                // .haystack_flatten(true)
                .dedupe(crate::Dedupe::None)
                .build(),
            needle: &AND_GATE,
            haystack: &AND_GATE_HAYSTACK,
            expected_matches: 2,
        },
        TestCase {
            name: "and_gate_self_dedupe_all",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                // .haystack_flatten(true)
                .dedupe(crate::Dedupe::All)
                .build(),
            needle: &AND_GATE,
            haystack: &AND_GATE_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "small_and_seq_3_and_tree_4_dedupe_none",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                // .haystack_flatten(true)
                .dedupe(crate::Dedupe::None)
                .build(),
            needle: &SMALL_AND_SEQ,
            haystack: &SMALL_AND_TREE_HAYSTACK,
            expected_matches: 4,
        },
        TestCase {
            name: "small_and_seq_3_and_tree_4_dedupe_all",
            config: Config::builder()
                .match_length(crate::config::MatchLength::Exact)
                // .haystack_flatten(true)
                .dedupe(crate::Dedupe::All)
                .build(),
            needle: &SMALL_AND_SEQ,
            haystack: &SMALL_AND_TREE_HAYSTACK,
            expected_matches: 2,
        },
        TestCase {
            name: "not_gate_self_dedupe_none",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::None)
                .build(),
            needle: &NOT_GATE,
            haystack: &NOT_GATE_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "not_gate_self_dedupe_all",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &NOT_GATE,
            haystack: &NOT_GATE_HAYSTACK,
            expected_matches: 1,
        },

        // --------------
        // examples/fixtures/basic/ff/verilog/dff_loop_toggle.v
        // --------------
        TestCase {
            name: "dff_loop_toggle_to_dff_loop_toggle",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &DFF_LOOP_TOGGLE,
            haystack: &DFF_LOOP_TOGGLE_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "dff_loop_toggle_to_dff_not",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &DFF_LOOP_TOGGLE,
            haystack: &DFF_NOT_HAYSTACK,
            expected_matches: 0,
        },
        TestCase {
            name: "dff_loop_toggle_to_not_dff",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &DFF_LOOP_TOGGLE,
            haystack: &NOT_DFF_HAYSTACK,
            expected_matches: 0,
        },
        TestCase {
            name: "dff_not_to_dff_loop_toggle",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &DFF_NOT,
            haystack: &DFF_LOOP_TOGGLE_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "dff_not_to_dff_not",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &DFF_NOT,
            haystack: &DFF_NOT_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "dff_not_to_not_dff",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &DFF_NOT,
            haystack: &NOT_DFF_HAYSTACK,
            expected_matches: 0,
        },
        TestCase {
            name: "not_dff_to_dff_loop_toggle",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &NOT_DFF,
            haystack: &DFF_LOOP_TOGGLE_HAYSTACK,
            expected_matches: 1,
        },
        TestCase {
            name: "not_dff_to_dff_not",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &NOT_DFF,
            haystack: &DFF_NOT_HAYSTACK,
            expected_matches: 0,
        },
        TestCase {
            name: "not_dff_to_not_dff",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &NOT_DFF,
            haystack: &NOT_DFF_HAYSTACK,
            expected_matches: 1,
        },
    ];


    pub static ref SECURITY_TEST_CASES: Vec<TestCase> = vec![
        // From many_locked_regs.v there are exactly 2 instances of each:
        // 2 × async_en, 2 × async_mux, 2 × sync_en, 2 × sync_mux

        // TestCase {
        //     name: "async_en_in_many_locked_regs",
        //     config: Config::builder().match_length(crate::MatchLength::Exact).build(),
        //     needle: &ASYNC_EN_IL,
        //     haystack: &MANY_LOCKED_REGS_IL,
        //     expected_matches: 2,
        // },
        // TestCase {
        //     name: "async_mux_in_many_locked_regs",
        //     config: Config::builder().match_length(crate::MatchLength::Exact).build(),
        //     needle: &ASYNC_MUX_IL,
        //     haystack: &MANY_LOCKED_REGS_IL,
        //     expected_matches: 2,
        // },
    ];


    pub static ref COMPOSITE_TEST_CASES: Vec<TestCase> = vec![
        // TestCase {
        //     name: "sdffe_then_and_simple_dedupe_none",
        //     config: Config::builder().match_length(crate::MatchLength::Exact).dedupe(crate::Dedupe::None).build(),
        //     needle: &SDFFE_THEN_AND,
        //     haystack: &AND_Q_DOUBLE_SDFFE,
        //     expected_matches: 4,
        // },

        // Base case: single AND (depth 1 only)
        TestCase {
            name: "rec_and_single_dedupe_none",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::None)
                .build(),
            needle: &REC_AND,
            haystack: &SINGLE_AND_HAYSTACK,
            expected_matches: 2,  // Just the base AND and the port permuations
        },
        TestCase {
            name: "rec_and_single_dedupe_all",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &REC_AND,
            haystack: &SINGLE_AND_HAYSTACK,
            expected_matches: 1,  // Dedupe doesn't change for single
        },

        // Sequential chain (N=3): 2 ANDs, expect 2 base + 1 depth2 = 3
        TestCase {
            name: "rec_and_seq_chain_dedupe_none",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::None)
                .build(),
            needle: &REC_AND,
            haystack: &AND_2_SEQ_HAYSTACK,
            expected_matches: 8,
        },
        TestCase {
            name: "rec_and_seq_chain_dedupe_inner",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::Inner)
                .build(),
            needle: &REC_AND,
            haystack: &AND_2_SEQ_HAYSTACK,
            expected_matches: 3,
        },

        TestCase {
            name: "rec_or_single_dedupe_none",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::None)
                .build(),
            needle: &REC_OR,
            haystack: &SINGLE_OR_HAYSTACK,
            expected_matches: 2,  // Just the base OR and port permutations
        },
        TestCase {
            name: "rec_or_single_dedupe_all",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::All)
                .build(),
            needle: &REC_OR,
            haystack: &SINGLE_OR_HAYSTACK,
            expected_matches: 1,  // Dedupe doesn't change for single
        },

        // Sequential chain (N=3): 2 ORs, expect 2 base + 1 depth2 = 3
        TestCase {
            name: "rec_or_seq_chain_dedupe_none",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::None)
                .build(),
            needle: &REC_OR,
            haystack: &OR_2_SEQ_HAYSTACK,
            expected_matches: 8,  // Adjusted for permutations (similar to AND)
        },
        TestCase {
            name: "rec_or_seq_chain_dedupe_inner",
            config: Config::builder()
                .match_length(MatchLength::Exact)
                .dedupe(Dedupe::Inner)
                .build(),
            needle: &REC_OR,
            haystack: &OR_2_SEQ_HAYSTACK,
            expected_matches: 3,
        },

    ];


    pub static ref VARIANT_TEST_CASES: Vec<TestCase> = vec![
        TestCase {
            name: "and_any_in_mixed_tree_dedupe_all",
            config: Config::builder().match_length(crate::MatchLength::Exact).dedupe(crate::Dedupe::All).build(),
            needle: &AND_ANY,
            haystack: &MIXED_AND_TREE,
            expected_matches: 7,  // 3 gates + 2 muxes + 2 nors
        },
    ];

    pub static ref ALL_TEST_CASES: Vec<TestCase> = {
        let mut all = BASIC_TEST_CASES.clone();
        all.extend(SECURITY_TEST_CASES.clone());
        all.extend(COMPOSITE_TEST_CASES.clone());
        all.extend(VARIANT_TEST_CASES.clone());
        all
    };
}
