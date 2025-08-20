mod integration_tests {
    #[cfg(test)]
    mod and {

        use rstest::rstest;
        use svql_driver::prelude::{DesignKey, Driver};
        use svql_driver::util::{ensure_loaded, new_shared_driver};

        lazy_static::lazy_static! {
            static ref DRIVER: Driver = new_shared_driver().unwrap();

            static ref AND_Q_DOUBLE_SDFFE: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v").unwrap();
            static ref AND_GATE: DesignKey = ensure_loaded(&DRIVER, "examples/patterns/basic/and/verilog/and_gate.v").unwrap();
            static ref AND_TREE: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/and/verilog/and_tree.v").unwrap();
            static ref AND_SEQ: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/and/verilog/and_seq.v").unwrap();

            static ref CONFIG: svql_subgraph::config::Config = svql_subgraph::config::Config::builder()
                .match_length(false)
                .none()
                .build();
        }

        #[rstest]
        // AND_Q_DOUBLE_SDFFE Needle
        #[case(&*AND_Q_DOUBLE_SDFFE, &*AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*AND_GATE, 0)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*AND_TREE, 0)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*AND_SEQ, 0)]
        // AND_GATE Needle
        #[case(&*AND_GATE, &*AND_Q_DOUBLE_SDFFE, 1)]
        #[case(&*AND_GATE, &*AND_GATE, 1)]
        #[case(&*AND_GATE, &*AND_TREE, 7)]
        #[case(&*AND_GATE, &*AND_SEQ, 7)]
        // AND_TREE Needle
        #[case(&*AND_TREE, &*AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&*AND_TREE, &*AND_GATE, 0)]
        #[case(&*AND_TREE, &*AND_TREE, 1)]
        #[case(&*AND_TREE, &*AND_SEQ, 0)]
        // AND_SEQ Needle
        #[case(&*AND_SEQ, &*AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&*AND_SEQ, &*AND_GATE, 0)]
        #[case(&*AND_SEQ, &*AND_TREE, 0)]
        #[case(&*AND_SEQ, &*AND_SEQ, 1)]
        fn test_subgraph_matches(
            #[case] needle: &'static DesignKey,
            #[case] haystack: &'static DesignKey,
            #[case] expected: usize,
        ) {
            let matches = svql_subgraph::find_subgraphs(
                DRIVER.get(needle).unwrap().as_ref(),
                DRIVER.get(haystack).unwrap().as_ref(),
                &CONFIG,
            );
            assert_eq!(
                matches.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                needle.top,
                haystack.top,
                matches.len()
            );
        }
    }
}
