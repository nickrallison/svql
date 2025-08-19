mod integration_tests {
    #[cfg(test)]
    mod and {

        use rstest::rstest;
        use svql_driver::prelude::Driver;
        use svql_driver::util::load_driver_from;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::queries::netlist::basic::and::And;

        lazy_static::lazy_static! {

            static ref AND_Q_DOUBLE_SDFFE: Driver = load_driver_from("examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v").unwrap();
            static ref AND_GATE: Driver = load_driver_from("examples/patterns/basic/and/verilog/and_gate.v").unwrap();
            static ref AND_TREE: Driver = load_driver_from("examples/fixtures/basic/and/verilog/and_tree.v").unwrap();
            static ref AND_SEQ: Driver = load_driver_from("examples/fixtures/basic/and/verilog/and_seq.v").unwrap();
        }

        fn root_instance() -> Instance {
            Instance::root("and".to_string())
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
            use svql_query::{Search, queries::netlist::basic::and::And};

            let hits = And::<Search>::query(needle, haystack, root_instance());

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
        fn and_bindings_present_and_gate_vs_and_tree() {
            // For each match, confirm a, b, y have bound design cells
            let hits = And::<Search>::query(&*AND_GATE, &*AND_TREE, root_instance());
            assert!(!hits.is_empty());

            for h in &hits {
                let a = h.a.val.as_ref().expect("missing a");
                let b = h.b.val.as_ref().expect("missing b");
                let y = h.y.val.as_ref().expect("missing y");

                assert!(
                    a.pat_cell_ref.is_some(),
                    "pattern cell for a should be present"
                );
                assert!(
                    b.pat_cell_ref.is_some(),
                    "pattern cell for b should be present"
                );
                assert!(
                    y.pat_cell_ref.is_some(),
                    "pattern cell for y should be present"
                );

                assert!(
                    a.design_cell_ref.is_some(),
                    "design source for a should be bound"
                );
                assert!(
                    b.design_cell_ref.is_some(),
                    "design source for b should be bound"
                );
                assert!(
                    y.design_cell_ref.is_some(),
                    "design driver for y should be bound"
                );
            }
        }

        fn any_connection_exists(
            hits: &[svql_query::queries::netlist::basic::and::And<svql_query::Match>],
        ) -> bool {
            for left in hits {
                if let Some(lhs_y_cell) = left.y.val.as_ref().and_then(|m| m.design_cell_ref) {
                    let lhs_net = lhs_y_cell.output()[0];
                    for right in hits {
                        if let Some(rhs_a_cell) =
                            right.a.val.as_ref().and_then(|m| m.design_cell_ref)
                        {
                            if lhs_net == rhs_a_cell.output()[0] {
                                return true;
                            }
                        }
                        if let Some(rhs_b_cell) =
                            right.b.val.as_ref().and_then(|m| m.design_cell_ref)
                        {
                            if lhs_net == rhs_b_cell.output()[0] {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }

        #[test]
        fn and_connectivity_exists_in_and_tree() {
            let hits = And::<Search>::query(&*AND_GATE, &*AND_TREE, root_instance());
            assert_eq!(hits.len(), 7, "sanity: expect 7 hits");

            let connected = any_connection_exists(&hits);
            assert!(
                connected,
                "expected at least one y->(a|b) connection among matches"
            );
        }

        #[test]
        fn and_connectivity_exists_in_and_seq() {
            let hits = And::<Search>::query(&*AND_GATE, &*AND_SEQ, root_instance());
            assert_eq!(hits.len(), 7, "sanity: expect 7 hits");

            let connected = any_connection_exists(&hits);
            assert!(
                connected,
                "expected at least one y->(a|b) connection among matches"
            );
        }
    }
}
