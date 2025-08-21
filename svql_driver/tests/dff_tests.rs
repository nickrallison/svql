mod integration_tests {
    #[cfg(test)]
    mod dff {
        use std::{path::PathBuf, sync::Arc};

        use prjunnamed_netlist::Design;
        use rstest::rstest;
        use svql_driver::prelude::Driver;

        lazy_static::lazy_static! {

            static ref DRIVER: Driver = Driver::new_workspace().unwrap();

            static ref COMB_D_DOUBLE_SDFFE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v", "comb_d_double_sdffe".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v"));
            static ref AND_Q_DOUBLE_SDFFE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v", "and_q_double_sdffe".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v"));
            static ref PAR_DOUBLE_SDFFE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/ff/verilog/par_double_sdffe.v", "par_double_sdffe".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/ff/verilog/par_double_sdffe.v"));
            static ref SEQ_DOUBLE_SDFFE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v", "seq_double_sdffe".to_string()).unwrap(), PathBuf::from("examples/fixtures/basic/ff/verilog/seq_double_sdffe.v"));
            static ref SDFFE: (Arc<Design>, PathBuf) = (DRIVER.get("examples/patterns/basic/ff/verilog/sdffe.v", "sdffe".to_string()).unwrap(), PathBuf::from("examples/patterns/basic/ff/verilog/sdffe.v"));

            static ref CONFIG: svql_subgraph::config::Config = svql_subgraph::config::Config::builder()
                .match_length(false)
                .none()
                .build();
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
            #[case] needle: &(Arc<Design>, PathBuf),
            #[case] haystack: &(Arc<Design>, PathBuf),
            #[case] expected: usize,
        ) {
            let matches =
                svql_subgraph::find_subgraphs(needle.0.as_ref(), haystack.0.as_ref(), &CONFIG);
            assert_eq!(
                matches.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                needle.1.display(),
                haystack.1.display(),
                matches.len()
            );
        }
    }
}
