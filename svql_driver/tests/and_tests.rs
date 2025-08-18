
mod integration_tests {
    #[cfg(test)]
    mod dff {
        use std::path::{Path, PathBuf};

        use rstest::rstest;
        use svql_driver::Driver;
        use svql_driver::util::load_driver_from;

        lazy_static::lazy_static! {

            static ref AND_Q_DOUBLE_SDFFE: (Driver, String) = load_driver_from("examples/patterns/basic/ff/and_q_double_sdffe.v");
            static ref AND_GATE: (Driver, String) = load_driver_from("examples/patterns/basic/and/and_gate.v");
            static ref AND_TREE: (Driver, String) = load_driver_from("examples/patterns/basic/and/and_tree.v");
            static ref AND_SEQ: (Driver, String) = load_driver_from("examples/patterns/basic/and/and_seq.v");
        }

    
        #[rstest]
        // AND_Q_DOUBLE_SDFFE Needle
        #[case(&AND_Q_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&AND_Q_DOUBLE_SDFFE, &AND_GATE, 0)]
        #[case(&AND_Q_DOUBLE_SDFFE, &AND_TREE, 0)]
        #[case(&AND_Q_DOUBLE_SDFFE, &AND_SEQ, 0)]
        // AND_GATE Needle
        #[case(&AND_GATE, &AND_Q_DOUBLE_SDFFE, 1)]
        #[case(&AND_GATE, &AND_GATE, 1)]
        #[case(&AND_GATE, &AND_TREE, 7)]
        #[case(&AND_GATE, &AND_SEQ, 7)]
        // AND_TREE Needle
        #[case(&AND_TREE, &AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&AND_TREE, &AND_GATE, 0)]
        #[case(&AND_TREE, &AND_TREE, 1)]
        #[case(&AND_TREE, &AND_SEQ, 0)]
        // AND_SEQ Needle
        #[case(&AND_SEQ, &AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&AND_SEQ, &AND_GATE, 0)]
        #[case(&AND_SEQ, &AND_TREE, 0)]
        #[case(&AND_SEQ, &AND_SEQ, 1)]
        fn test_subgraph_matches(
            #[case] needle_tuple: &'static (Driver, String),
            #[case] haystack_tuple: &'static (Driver, String),
            #[case] expected: usize,
        ) {
            let (needle, needle_name) = needle_tuple;
            let (haystack, haystack_name) = haystack_tuple;
            let matches = svql_driver::subgraph::find_subgraphs(needle.design_as_ref(), haystack.design_as_ref());
            assert_eq!(matches.len(), expected, "Expected {} matches for needle {}, against haystack {}, got {}", expected, &needle_name, &haystack_name, matches.len());
        }
    }
}