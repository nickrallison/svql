mod integration_tests {
    #[cfg(test)]
    mod dff {
        use rstest::rstest;
        use svql_driver::prelude::Driver;
        use svql_driver::util::load_driver_from;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::queries::netlist::dff::Sdffe;

        lazy_static::lazy_static! {
            static ref COMB_D_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/comb_d_double_sdffe.v").unwrap();
            static ref AND_Q_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/and_q_double_sdffe.v").unwrap();
            static ref PAR_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/par_double_sdffe.v").unwrap();
            static ref SEQ_DOUBLE_SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/seq_double_sdffe.v").unwrap();
            static ref SDFFE: Driver = load_driver_from("examples/patterns/basic/ff/sdffe.v").unwrap();
        }

        fn root_instance() -> Instance {
            Instance::root("sdffe".to_string())
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
        fn test_sdffe_subgraph_matches(
            #[case] needle: &'static Driver,
            #[case] haystack: &'static Driver,
            #[case] expected: usize,
        ) {
            let hits = Sdffe::<Search>::query(needle, haystack, root_instance());
            assert_eq!(
                hits.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                needle.module_name(),
                haystack.module_name(),
                hits.len()
            );
        }

        #[test]
        fn sdffe_bindings_present_in_seq_double() {
            // sanity: two flops connected in sequence
            let hits = Sdffe::<Search>::query(&*SDFFE, &*SEQ_DOUBLE_SDFFE, root_instance());
            assert_eq!(
                hits.len(),
                2,
                "sanity: expect 2 sdffe matches in seq_double_sdffe"
            );

            for h in &hits {
                let clk = h.clk.val.as_ref().expect("missing clk");
                let d = h.d.val.as_ref().expect("missing d");
                let reset = h.reset.val.as_ref().expect("missing reset");
                let q = h.q.val.as_ref().expect("missing q");

                assert!(
                    clk.pat_cell_ref.is_some(),
                    "pattern cell for clk should be present"
                );
                assert!(
                    d.pat_cell_ref.is_some(),
                    "pattern cell for d should be present"
                );
                assert!(
                    reset.pat_cell_ref.is_some(),
                    "pattern cell for reset should be present"
                );
                assert!(
                    q.pat_cell_ref.is_some(),
                    "pattern cell for q should be present"
                );

                assert!(
                    clk.design_cell_ref.is_some(),
                    "design source for clk should be bound"
                );
                assert!(
                    d.design_cell_ref.is_some(),
                    "design source for d should be bound"
                );
                assert!(
                    reset.design_cell_ref.is_some(),
                    "design source for reset should be bound"
                );
                assert!(
                    q.design_cell_ref.is_some(),
                    "design driver for q should be bound"
                );
            }
        }

        fn any_q_to_d_connection(
            hits: &[svql_query::queries::netlist::dff::Sdffe<svql_query::Match>],
        ) -> bool {
            // Compare the Q net of one hit to the D source net of another.
            // Our Match only carries the design cell, but these are 1-bit nets
            // in the provided examples so using bit 0 is sufficient.
            for left in hits {
                if let Some(lhs_q_cell) = left.q.val.as_ref().and_then(|m| m.design_cell_ref) {
                    let q_net = lhs_q_cell.output()[0];
                    for right in hits {
                        if let Some(rhs_d_src) =
                            right.d.val.as_ref().and_then(|m| m.design_cell_ref)
                        {
                            if q_net == rhs_d_src.output()[0] {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        #[test]
        fn sdffe_connectivity_exists_in_seq_double() {
            let hits = Sdffe::<Search>::query(&*SDFFE, &*SEQ_DOUBLE_SDFFE, root_instance());
            assert_eq!(hits.len(), 2, "sanity: expect 2 sdffe hits");

            let connected = any_q_to_d_connection(&hits);
            assert!(
                connected,
                "expected at least one connection where q of one flop drives d of the other"
            );
        }
    }
}
