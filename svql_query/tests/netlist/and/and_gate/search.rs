// svql_query/tests/netlist/and/and_gate/search.rs
mod integration_tests {
    #[cfg(test)]
    mod and {

        use rstest::rstest;
        use svql_driver::driver::Driver;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::netlist::SearchableNetlist;
        use svql_query::queries::netlist::basic::and::and_gate::AndGate;
        use svql_subgraph::config::Config;

        lazy_static::lazy_static! {
            static ref DRIVER: Driver = Driver::new_workspace().expect("Failed to create driver");
            static ref CONFIG: Config = Config::builder().exact_length().none().build();
        }

        fn root_instance() -> Instance {
            Instance::root("and".to_string())
        }

        #[rstest]
        #[case(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            2
        )]
        #[case(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "examples/patterns/basic/and/verilog/and_gate.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "examples/fixtures/basic/and/verilog/and_tree.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "examples/fixtures/basic/and/verilog/and_seq.v",
            0
        )]
        #[case(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            1
        )]
        #[case(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "examples/patterns/basic/and/verilog/and_gate.v",
            1
        )]
        #[case(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "examples/fixtures/basic/and/verilog/and_tree.v",
            7
        )]
        #[case(
            "examples/patterns/basic/and/verilog/and_gate.v",
            "examples/fixtures/basic/and/verilog/and_seq.v",
            7
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_tree.v",
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_tree.v",
            "examples/patterns/basic/and/verilog/and_gate.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_tree.v",
            "examples/fixtures/basic/and/verilog/and_tree.v",
            1
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_tree.v",
            "examples/fixtures/basic/and/verilog/and_seq.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "examples/patterns/basic/and/verilog/and_gate.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "examples/fixtures/basic/and/verilog/and_tree.v",
            0
        )]
        #[case(
            "examples/fixtures/basic/and/verilog/and_seq.v",
            "examples/fixtures/basic/and/verilog/and_seq.v",
            1
        )]
        fn test_subgraph_matches(
            #[case] needle_path: &str,
            #[case] haystack_path: &str,
            #[case] expected: usize,
        ) {
            let haystack_key = svql_driver::DriverKey::new(haystack_path, {
                let path = std::path::Path::new(haystack_path);
                path.file_stem().unwrap().to_str().unwrap().to_string()
            });

            let context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");
            let hits = AndGate::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);

            assert_eq!(
                hits.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                needle_path,
                haystack_path,
                hits.len()
            );
        }

        #[test]
        fn and_bindings_present_and_gate_vs_and_tree() {
            let needle_key = svql_driver::DriverKey::new(
                "examples/patterns/basic/and/verilog/and_gate.v",
                "and_gate".to_string(),
            );
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/verilog/and_tree.v",
                "and_tree".to_string(),
            );

            let context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");
            let hits = AndGate::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
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
            hits: &[svql_query::queries::netlist::basic::and::and_gate::AndGate<
                svql_query::Match,
            >],
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
            let needle_key = svql_driver::DriverKey::new(
                "examples/patterns/basic/and/verilog/and_gate.v",
                "and_gate".to_string(),
            );
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/verilog/and_tree.v",
                "and_tree".to_string(),
            );

            let context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");
            let hits = AndGate::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
            assert_eq!(hits.len(), 7, "sanity: expect 7 hits");

            let connected = any_connection_exists(&hits);
            assert!(
                connected,
                "expected at least one y->(a|b) connection among matches"
            );
        }

        #[test]
        fn and_connectivity_exists_in_and_seq() {
            let needle_key = svql_driver::DriverKey::new(
                "examples/patterns/basic/and/verilog/and_gate.v",
                "and_gate".to_string(),
            );
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/verilog/and_seq.v",
                "and_seq".to_string(),
            );

            let context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");
            let hits = AndGate::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
            assert_eq!(hits.len(), 7, "sanity: expect 7 hits");

            let connected = any_connection_exists(&hits);
            assert!(
                connected,
                "expected at least one y->(a|b) connection among matches"
            );
        }
    }
}
