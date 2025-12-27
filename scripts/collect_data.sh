export RUST_LOG=info 

/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/cv32e40p_fp_wrapper_netlist.json cv32e40p_fp_wrapper 3
/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/soc_interconnect_wrap_netlist.json soc_interconnect_wrap 2
/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json soc_peripherals 3
/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json udma_core 3

/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json tile 3