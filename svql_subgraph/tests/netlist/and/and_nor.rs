mod integration_tests {

    #[cfg(test)]
    mod and_nor {
        use prjunnamed_netlist::Design;
        use svql_subgraph::{config::Config, find_subgraphs, util::load_design_from};

        lazy_static::lazy_static! {
            static ref MIXED_AND_TREE: Design = load_design_from("examples/fixtures/basic/and/json/mixed_and_tree.json").unwrap();
            static ref AND_NOR: Design = load_design_from("examples/patterns/basic/and/json/and_nor.json").unwrap();
            static ref CONFIG_DEDUPE_FULL: Config = Config::builder().exact_length().full().build();
            static ref CONFIG_DEDUPE_GATES_ONLY: Config = Config::builder().exact_length().gates_only().build();
        }

        #[test]
        fn and_nor_matches_in_mixed_tree_gates_only() {
            // mixed_and_tree has 2 and_nor instances
            let hits = find_subgraphs(&*AND_NOR, &*MIXED_AND_TREE, &CONFIG_DEDUPE_GATES_ONLY);
            assert_eq!(
                hits.len(),
                2,
                "expected 2 and_nor matches in mixed_and_tree"
            );
        }

        #[test]
        fn and_nor_matches_in_mixed_tree_full() {
            // mixed_and_tree has 2 and_nor instances
            let hits = find_subgraphs(&*AND_NOR, &*MIXED_AND_TREE, &CONFIG_DEDUPE_FULL);
            assert_eq!(
                hits.len(),
                4,
                "expected 4 and_nor matches in mixed_and_tree"
            );
        }
    }
}
