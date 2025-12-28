export RUST_LOG=info 

/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v cv32e40p_fp_wrapper 3 false
/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v soc_interconnect_wrap 3 false
/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json soc_peripherals 2 true
/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json udma_core 3 true

/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json tile 3 true

/bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json e203_soc_top 3 true