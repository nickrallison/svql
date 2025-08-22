// svql_query/tests/netlist/and/and_mux.rs
mod integration_tests {
    #[cfg(test)]
    mod and_mux {
        use svql_common::Config;
        use svql_driver::driver::Driver;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::netlist::SearchableNetlist;
        use svql_query::queries::netlist::basic::and::AndMux;

        lazy_static::lazy_static! {
            static ref DRIVER: Driver = Driver::new_workspace().expect("Failed to create driver");
            static ref CONFIG: Config = Config::builder().exact_length().none().build();
        }

        fn root_instance() -> Instance {
            Instance::root("and_mux".to_string())
        }

        #[test]
        fn and_mux_matches_in_mixed_tree() {
            // Get the base context with just the pattern
            let mut context = AndMux::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/json/mixed_and_tree.json",
                "mixed_and_tree".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/and/json/mixed_and_tree.json",
                    "mixed_and_tree".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

            // mixed_and_tree has 2 and_mux instances
            let hits = AndMux::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
            assert_eq!(
                hits.len(),
                2,
                "expected 2 and_mux matches in mixed_and_tree"
            );
        }

        #[test]
        fn and_mux_bindings_present_in_mixed_tree() {
            // Get the base context with just the pattern
            let mut context = AndMux::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/and/json/mixed_and_tree.json",
                "mixed_and_tree".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/and/json/mixed_and_tree.json",
                    "mixed_and_tree".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

            let hits = AndMux::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
            assert!(!hits.is_empty(), "expected and_mux hits in mixed_and_tree");

            for h in &hits {
                let a = h.a.val.as_ref().expect("missing a binding");
                let b = h.b.val.as_ref().expect("missing b binding");
                let y = h.y.val.as_ref().expect("missing y binding");

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
    }
}
