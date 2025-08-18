mod integration_tests {
    #[cfg(test)]
    mod dff {
        use rstest::rstest;
        use svql_driver::prelude::Driver;
        use svql_driver::util::load_driver_from;

        lazy_static::lazy_static! {

            static ref COMB_D_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/comb_d_double_sdffe.v").unwrap();
            static ref AND_Q_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/and_q_double_sdffe.v").unwrap();
            static ref PAR_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/par_double_sdffe.v").unwrap();
            static ref SEQ_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/seq_double_sdffe.v").unwrap();
            static ref SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/sdffe.v").unwrap();
        }

        #[rstest]
        // Comb D Needle
        #[case(&COMB_D_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 2)]
        #[case(&COMB_D_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&COMB_D_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 0)]
        #[case(&COMB_D_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 0)]
        #[case(&COMB_D_DOUBLE_SDFFE, &SDFFE, 0)]
        // And Q Needle
        #[case(&AND_Q_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 0)]
        #[case(&AND_Q_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&AND_Q_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 0)]
        #[case(&AND_Q_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 0)]
        #[case(&AND_Q_DOUBLE_SDFFE, &SDFFE, 0)]
        // Par Double Needle
        #[case(&PAR_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 2)]
        #[case(&PAR_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&PAR_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 2)]
        #[case(&PAR_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 2)]
        #[case(&PAR_DOUBLE_SDFFE, &SDFFE, 0)]
        // Seq Double Needle
        #[case(&SEQ_DOUBLE_SDFFE, &COMB_D_DOUBLE_SDFFE, 0)]
        #[case(&SEQ_DOUBLE_SDFFE, &AND_Q_DOUBLE_SDFFE, 0)]
        #[case(&SEQ_DOUBLE_SDFFE, &PAR_DOUBLE_SDFFE, 0)]
        #[case(&SEQ_DOUBLE_SDFFE, &SEQ_DOUBLE_SDFFE, 1)]
        #[case(&SEQ_DOUBLE_SDFFE, &SDFFE, 0)]
        // SDFFE Needle
        #[case(&SDFFE, &COMB_D_DOUBLE_SDFFE, 2)]
        #[case(&SDFFE, &AND_Q_DOUBLE_SDFFE, 2)]
        #[case(&SDFFE, &PAR_DOUBLE_SDFFE, 2)]
        #[case(&SDFFE, &SEQ_DOUBLE_SDFFE, 2)]
        #[case(&SDFFE, &SDFFE, 1)]
        fn test_subgraph_matches(
            #[case] needle: &'static Driver,
            #[case] haystack: &'static Driver,
            #[case] expected: usize,
        ) {
            let matches =
                svql_subgraph::find_subgraphs(needle.design_as_ref(), haystack.design_as_ref());
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
