# export RUST_LOG=collect_data=info,svql_subgraph=debug,svql_query=debug
export RUST_LOG=error

# /bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v cv32e40p_fp_wrapper 3 false
# /bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v soc_interconnect_wrap 3 false
# /bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json soc_peripherals 2 true
# /bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json udma_core 3 true

# /bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json tile 2 true

# /bin/time cargo run --bin example_query --release --features svql_subgraph/rayon --features svql_query/parallel -- examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json e203_soc_top 3 true


mkdir -p bin
# cargo run --bin collect_data --release \
#     --features svql_subgraph/rayon \
#     --features svql_query/parallel \
#     -- --config scripts/collect_data.json --format csv > bin/results.txt

cargo run --bin collect_data --release \
    --features svql_subgraph/rayon \
    --features svql_query/parallel \
    -- --config scripts/collect_data.json --format pretty >> bin/results_par.txt

cargo run --bin collect_data --release \
    -- --config scripts/collect_data.json --format pretty >> bin/results_single_threaded.txt

# cargo run --bin collect_data --release \
#     --features svql_subgraph/rayon \
#     --features svql_query/parallel \
#     -- --config scripts/collect_data.json --format csv > bin/results.csv



# RUST_LOG=debug cargo run --bin collect_data --release \
#     --features svql_subgraph/rayon \
#     --features svql_query/parallel \
#     -- --config scripts/collect_data.json --format pretty

