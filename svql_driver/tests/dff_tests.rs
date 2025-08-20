mod integration_tests {
    #[cfg(test)]
    mod dff {
        use rstest::rstest;
        use svql_driver::prelude::{DesignKey, Driver};
        use svql_driver::util::{ensure_loaded, new_shared_driver};

        lazy_static::lazy_static! {
            static ref DRIVER: Driver = new_shared_driver().unwrap();

            static ref COMB_D_DOUBLE_SDFFE: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v").unwrap();
            static ref AND_Q_DOUBLE_SDFFE: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v").unwrap();
            static ref PAR_DOUBLE_SDFFE: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/ff/verilog/par_double_sdffe.v").unwrap();
            static ref SEQ_DOUBLE_SDFFE: DesignKey = ensure_loaded(&DRIVER, "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v").unwrap();
            static ref SDFFE: DesignKey = ensure_loaded(&DRIVER, "examples/patterns/basic/ff/verilog/sdffe.v").unwrap();

            //
            static ref CONFIG: svql_subgraph::config::Config = svql_subgraph::config::Config::builder()
                .exact_length()
                .none()
                .build();
        }

        #[rstest]
        // Comb D Needle
        #[case(&*COMB_D_DOUBLE_SDFFE, &*COMB_D_DOUBLE_SDFFE, 2)]
        #[case(&*COMB_D_DOUBLE_SDFFE, &*AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&*COMB_D_DOUBLE_SDFFE, &*PAR_DOUBLE_SDFFE, 0)]
        #[case(&*COMB_D_DOUBLE_SDFFE, &*SEQ_DOUBLE_SDFFE, 0)]
        #[case(&*COMB_D_DOUBLE_SDFFE, &*SDFFE, 0)]
        // And Q Needle
        #[case(&*AND_Q_DOUBLE_SDFFE, &*COMB_D_DOUBLE_SDFFE, 0)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*PAR_DOUBLE_SDFFE, 0)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*SEQ_DOUBLE_SDFFE, 0)]
        #[case(&*AND_Q_DOUBLE_SDFFE, &*SDFFE, 0)]
        // Par Double Needle
        #[case(&*PAR_DOUBLE_SDFFE, &*COMB_D_DOUBLE_SDFFE, 2)]
        #[case(&*PAR_DOUBLE_SDFFE, &*AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&*PAR_DOUBLE_SDFFE, &*PAR_DOUBLE_SDFFE, 2)]
        #[case(&*PAR_DOUBLE_SDFFE, &*SEQ_DOUBLE_SDFFE, 2)]
        #[case(&*PAR_DOUBLE_SDFFE, &*SDFFE, 0)]
        // Seq Double Needle
        #[case(&*SEQ_DOUBLE_SDFFE, &*COMB_D_DOUBLE_SDFFE, 0)]
        #[case(&*SEQ_DOUBLE_SDFFE, &*AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&*SEQ_DOUBLE_SDFFE, &*PAR_DOUBLE_SDFFE, 0)]
        #[case(&*SEQ_DOUBLE_SDFFE, &*SEQ_DOUBLE_SDFFE, 1)]
        #[case(&*SEQ_DOUBLE_SDFFE, &*SDFFE, 0)]
        // SDFFE Needle
        #[case(&*SDFFE, &*COMB_D_DOUBLE_SDFFE, 2)]
        #[case(&*SDFFE, &*AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&*SDFFE, &*PAR_DOUBLE_SDFFE, 2)]
        #[case(&*SDFFE, &*SEQ_DOUBLE_SDFFE, 2)]
        #[case(&*SDFFE, &*SDFFE, 1)]
        fn test_subgraph_matches(
            #[case] needle: &'static DesignKey,
            #[case] haystack: &'static DesignKey,
            #[case] expected: usize,
        ) {
            let pat_arc = DRIVER.get(needle).expect("pattern present");
            let hay_arc = DRIVER.get(haystack).expect("haystack present");

            let matches =
                svql_subgraph::find_subgraphs(pat_arc.as_ref(), hay_arc.as_ref(), &CONFIG);
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
