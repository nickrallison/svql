#!/bin/bash

# cargo run --profile release --package svql_cli --bin svql_cli -- \
#     -f examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v -m cv32e40p_fp_wrapper -\
#     -f examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json -m e203_soc_top \
#     -f examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v -m soc_interconnect_wrap \
#     -f examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json -m soc_peripherals \
#     -f examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json -m tile \
#     -f examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json -m udma_core \
#     --query Cwe1234 \
#     --query Cwe1271 \
#     --parallel \
#     --profile \
#     --output-csv results.csv

cargo run --profile release --package svql_cli --bin svql_cli -- \
    --design "examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v --module cv32e40p_fp_wrapper" \
    --design "examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v --module soc_interconnect_wrap" \
    --design "examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json --module soc_peripherals --raw" \
    --design "examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json --module udma_core --raw" \
    --design "examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json --module tile --raw" \
    --design "examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json --module e203_soc_top --raw" \
    --parallel \
    --profile \
    --output-csv bin/results.csv \
    --output-latex bin/results.tex

cargo run --profile release --package svql_cli --bin svql_cli -- \
    --design "examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v --module cv32e40p_fp_wrapper" \
    --parallel \
    --profile \
    --output-csv bin/results.csv \
    --output-latex bin/results.tex \
    -q cdc-violation
