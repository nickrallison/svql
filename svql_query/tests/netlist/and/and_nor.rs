mod integration_tests {
    #[cfg(test)]
    mod and_nor {
        use svql_driver::prelude::Driver;
        use svql_driver::util::load_driver_from;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::queries::netlist::basic::and::and_nor::AndNor;

        lazy_static::lazy_static! {
            static ref MIXED_AND_TREE: Driver = load_driver_from("examples/fixtures/basic/and/json/mixed_and_tree.json").unwrap();
            static ref AND_NOR: Driver = load_driver_from("examples/patterns/basic/and/verilog/and_nor.v").unwrap();
        }

        fn root_instance() -> Instance {
            Instance::root("and_nor".to_string())
        }

        #[test]
        fn and_nor_matches_in_mixed_tree() {
            // mixed_and_tree has 2 and_nor instances
            let hits = AndNor::<Search>::query(&*AND_NOR, &*MIXED_AND_TREE, root_instance());
            assert_eq!(
                hits.len(),
                2,
                "expected 2 and_nor matches in mixed_and_tree"
            );
        }

        #[test]
        fn and_nor_bindings_present_in_mixed_tree() {
            let hits = AndNor::<Search>::query(&*AND_NOR, &*MIXED_AND_TREE, root_instance());
            assert!(!hits.is_empty(), "expected and_nor hits in mixed_and_tree");

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
