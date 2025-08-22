// svql_query/tests/netlist/and/and_nor.rs
mod integration_tests {
    #[cfg(test)]
    mod and_nor {
        use svql_driver::driver::Driver;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::netlist::SearchableNetlist;
        use svql_query::queries::netlist::basic::and::AndNor;
        use svql_subgraph::config::Config;

        lazy_static::lazy_static! {
            static ref DRIVER: Driver = Driver::new_workspace().expect("Failed to create driver");
            static ref CONFIG_DEDUPE_NONE: Config = Config::builder().exact_length().none().build();
            static ref CONFIG_DEDUPE_AUTO_MORPH: Config = Config::builder().exact_length().auto_morph().build();
        }

        fn root_instance() -> Instance {
            Instance::root("and_nor".to_string())
        }

        #[test]
        fn and_nor_matches_in_mixed_tree_auto_morph() {
            // Get the base context with just the pattern
            let mut context = AndNor::<Search>::context(&*DRIVER).expect("Failed to get context");

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

            // mixed_and_tree has 2 and_nor instances
            let hits = AndNor::<Search>::query(
                &haystack_key,
                &context,
                root_instance(),
                &CONFIG_DEDUPE_AUTO_MORPH,
            );
            assert_eq!(
                hits.len(),
                2,
                "expected 2 and_nor matches in mixed_and_tree"
            );
        }

        #[test]
        fn and_nor_matches_in_mixed_tree_none() {
            // Get the base context with just the pattern
            let mut context = AndNor::<Search>::context(&*DRIVER).expect("Failed to get context");

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

            // mixed_and_tree has 2 and_nor instances
            let hits = AndNor::<Search>::query(
                &haystack_key,
                &context,
                root_instance(),
                &CONFIG_DEDUPE_NONE,
            );
            assert_eq!(
                hits.len(),
                4,
                "expected 4 and_nor matches in mixed_and_tree"
            );
        }
    }
}
