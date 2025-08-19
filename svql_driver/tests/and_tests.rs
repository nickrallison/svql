mod integration_tests {
    #[cfg(test)]
    mod and {

        use rstest::rstest;
        use svql_driver::prelude::Driver;
        use svql_driver::util::load_driver_from;

        lazy_static::lazy_static! {

            static ref AND_Q_DOUBLE_SDFFE: Driver = load_driver_from("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v").unwrap();
            static ref AND_GATE: Driver = load_driver_from("examples/patterns/basic/and/verilog/and_gate.v").unwrap();
            static ref AND_TREE: Driver = load_driver_from("examples/fixtures/basic/and/verilog/and_tree.v").unwrap();
            static ref AND_SEQ: Driver = load_driver_from("examples/fixtures/basic/and/verilog/and_seq.v").unwrap();

            //
            static ref CONFIG: svql_subgraph::config::Config = svql_subgraph::config::Config::builder()
                .match_length(false)
                .full()
                .build();
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
            #[case] needle: &'static Driver,
            #[case] haystack: &'static Driver,
            #[case] expected: usize,
        ) {
            let matches = svql_subgraph::find_subgraphs(
                needle.design_as_ref(),
                haystack.design_as_ref(),
                &CONFIG,
            );
            assert_eq!(
                matches.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                needle.module_name(),
                haystack.module_name(),
                matches.len()
            );
        }
    }
}
