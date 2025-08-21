// svql_query/tests/netlist/dff/search.rs
mod integration_tests {
    #[cfg(test)]
    mod dff {
        use rstest::rstest;
        use svql_driver::driver::Driver;
        use svql_query::Search;
        use svql_query::instance::Instance;
        use svql_query::netlist::{NetlistMeta, SearchableNetlist};
        use svql_query::queries::netlist::basic::dff::Sdffe;
        use svql_subgraph::config::Config;

        lazy_static::lazy_static! {
            static ref DRIVER: Driver = Driver::new_workspace().expect("Failed to create driver");
            static ref CONFIG: Config = Config::builder().exact_length().none().build();
        }

        fn root_instance() -> Instance {
            Instance::root("sdffe".to_string())
        }

        #[rstest]
        #[case(
            "examples/fixtures/basic/ff/verilog/comb_d_double_sdffe.v",
            "comb_d_double_sdffe",
            2
        )]
        #[case(
            "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v",
            "and_q_double_sdffe",
            2
        )]
        #[case(
            "examples/fixtures/basic/ff/verilog/par_double_sdffe.v",
            "par_double_sdffe",
            2
        )]
        #[case(
            "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
            "seq_double_sdffe",
            2
        )]
        #[case("examples/patterns/basic/ff/verilog/sdffe.v", "sdffe", 1)]
        fn test_sdffe_subgraph_matches(
            #[case] haystack_path: &str,
            #[case] haystack_module: &str,
            #[case] expected: usize,
        ) {
            // Get the base context with just the pattern
            let mut context = Sdffe::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let haystack_key =
                svql_driver::DriverKey::new(haystack_path, haystack_module.to_string());

            let (_, haystack_design) = DRIVER
                .get_or_load_design(haystack_path, haystack_module.to_string())
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

            let hits = Sdffe::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
            assert_eq!(
                hits.len(),
                expected,
                "Expected {} matches for needle {}, against haystack {}, got {}",
                expected,
                Sdffe::MODULE_NAME,
                haystack_path,
                hits.len()
            );
        }

        #[test]
        fn sdffe_bindings_present_in_seq_double() {
            // Get the base context with just the pattern
            let mut context = Sdffe::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let needle_key = svql_driver::DriverKey::new(
                "examples/patterns/basic/ff/verilog/sdffe.v",
                "sdffe".to_string(),
            );
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
                "seq_double_sdffe".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
                    "seq_double_sdffe".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

            // sanity: two flops connected in sequence
            let hits = Sdffe::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
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
            hits: &[svql_query::queries::netlist::basic::dff::Sdffe<svql_query::Match>],
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
            // Get the base context with just the pattern
            let mut context = Sdffe::<Search>::context(&*DRIVER).expect("Failed to get context");

            // Load the haystack design and add it to the context
            let needle_key = svql_driver::DriverKey::new(
                "examples/patterns/basic/ff/verilog/sdffe.v",
                "sdffe".to_string(),
            );
            let haystack_key = svql_driver::DriverKey::new(
                "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
                "seq_double_sdffe".to_string(),
            );

            let (_, haystack_design) = DRIVER
                .get_or_load_design(
                    "examples/fixtures/basic/ff/verilog/seq_double_sdffe.v",
                    "seq_double_sdffe".to_string(),
                )
                .expect("Failed to load haystack design");

            // Add the haystack design to the context
            context = context.with_design(haystack_key.clone(), haystack_design);

            let hits = Sdffe::<Search>::query(&haystack_key, &context, root_instance(), &CONFIG);
            assert_eq!(hits.len(), 2, "sanity: expect 2 sdffe hits");

            let connected = any_q_to_d_connection(&hits);
            assert!(
                connected,
                "expected at least one connection where q of one flop drives d of the other"
            );
        }
    }
}
