// svql_query/tests/netlist/and/and_gate/search.rs
mod integration_tests {
    #[cfg(test)]
    mod and {

        use rstest::rstest;
        use svql_driver::driver::Driver;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::netlist::{NetlistMeta, SearchableNetlist};
        use svql_query::queries::netlist::basic::and::AndGate;
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
            "and_q_double_sdffe",
            2
        )]
        #[case("examples/patterns/basic/and/verilog/and_gate.v", "and_gate", 1)]
        #[case("examples/fixtures/basic/and/verilog/and_tree.v", "and_tree", 7)]
        #[case("examples/fixtures/basic/and/verilog/and_seq.v", "and_seq", 7)]
        fn test_subgraph_matches(
            #[case] haystack_path: &str,
            #[case] haystack_module_name: &str,
            #[case] expected: usize,
        ) {
            // Get the base context with just the needle (pattern)
            let mut context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let haystack_key =
                svql_driver::DriverKey::new(haystack_path, haystack_module_name.to_string());
            let (_, haystack_design) = DRIVER
                .get_or_load_design(haystack_path, haystack_module_name.to_string())
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

            let hits = AndGate::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);

            assert_eq!(
                hits.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                AndGate::MODULE_NAME,
                haystack_path,
                hits.len()
            );
        }

        #[test]
        fn and_bindings_present_and_gate_vs_and_tree() {
            // Get the base context with just the needle (pattern)
            let mut context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/verilog/and_tree.v",
                "and_tree".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/and/verilog/and_tree.v",
                    "and_tree".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

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
            hits: &[svql_query::queries::netlist::basic::and::AndGate<svql_query::Match>],
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
            // Get the base context with just the needle (pattern)
            let mut context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/verilog/and_tree.v",
                "and_tree".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/and/verilog/and_tree.v",
                    "and_tree".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

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
            // Get the base context with just the needle (pattern)
            let mut context = AndGate::<Search>::context(&*DRIVER).expect("Failed to get context");
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/verilog/and_seq.v",
                "and_seq".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/and/verilog/and_seq.v",
                    "and_seq".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

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
