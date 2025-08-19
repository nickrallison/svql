mod integration_tests {
    #[cfg(test)]
    mod and_nor {
        use svql_driver::prelude::Driver;
        use svql_driver::util::load_driver_from;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::queries::netlist::basic::and::and_nor::AndNor;
        use svql_subgraph::config::{Config, DedupeMode};

        lazy_static::lazy_static! {
            static ref MIXED_AND_TREE: Driver = load_driver_from("examples/fixtures/basic/and/json/mixed_and_tree.json").unwrap();
            static ref AND_NOR: Driver = load_driver_from("examples/patterns/basic/and/verilog/and_nor.v").unwrap();
            static ref CONFIG_DEDUPE_FULL: Config = Config::new(true, DedupeMode::Full);
            static ref CONFIG_DEDUPE_GATES_ONLY: Config = Config::new(true, DedupeMode::GatesOnly);
        }

        fn root_instance() -> Instance {
            Instance::root("and_nor".to_string())
        }

        #[test]
        fn and_nor_matches_in_mixed_tree_gates_only() {
            // mixed_and_tree has 2 and_nor instances
            let hits = AndNor::<Search>::query(
                &*AND_NOR,
                &*MIXED_AND_TREE,
                root_instance(),
                &CONFIG_DEDUPE_GATES_ONLY,
            );
            assert_eq!(
                hits.len(),
                2,
                "expected 2 and_nor matches in mixed_and_tree"
            );
        }

        #[test]
        fn and_nor_matches_in_mixed_tree_full() {
            // mixed_and_tree has 2 and_nor instances
            let hits = AndNor::<Search>::query(
                &*AND_NOR,
                &*MIXED_AND_TREE,
                root_instance(),
                &CONFIG_DEDUPE_FULL,
            );
            assert_eq!(
                hits.len(),
                40,
                "expected 40 and_nor matches in mixed_and_tree"
            );
        }
    }
}
