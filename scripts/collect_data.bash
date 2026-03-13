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

# cargo run --profile release --package svql_cli --bin svql_cli -- \
#     --design "examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v --module cv32e40p_fp_wrapper" \
#     --design "examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v --module soc_interconnect_wrap" \
#     --design "examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json --module soc_peripherals --raw" \
#     --design "examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json --module udma_core --raw" \
#     --design "examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json --module tile --raw" \
#     --design "examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json --module e203_soc_top --raw" \
#     --parallel \
#     --profile \
#     --output-csv bin/results.csv \
#     --output-latex bin/results.tex

# cargo run --profile release --package svql_cli --bin svql_cli -- \
#     --design "examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v --module cv32e40p_fp_wrapper" \
#     --parallel \
#     --profile \
#     --output-csv bin/results.csv \
#     --output-latex bin/results.tex \
#     -q cdc-violation

#!/usr/bin/env bash
set -euo pipefail

OUT_DIR="bin/results"
mkdir -p "$OUT_DIR"

QUERIES=(
  "cwe1234"
  "cwe1271"
  "cwe1280"
)

DESIGNS=(
  "examples/fixtures/larger_designs/verilog/hackatdac18/cv32e40p_fp_wrapper_netlist.v --module cv32e40p_fp_wrapper"
  "examples/fixtures/larger_designs/verilog/hackatdac18/soc_interconnect_wrap_netlist.v --module soc_interconnect_wrap"
  "examples/fixtures/larger_designs/json/hackatdac18/soc_peripherals_netlist.json --module soc_peripherals --raw"
  "examples/fixtures/larger_designs/json/hackatdac18/udma_core_netlist.json --module udma_core --raw"
  "examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json --module tile --raw"
  "examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json --module e203_soc_top --raw"
)

for query in "${QUERIES[@]}"; do
  echo "============================================================"
  echo "Running query: $query"
  echo "============================================================"

  cmd=(
    cargo run
    --profile release
    --package svql_cli
    --bin svql_cli
    --
  )

  for design in "${DESIGNS[@]}"; do
    cmd+=(--design "$design")
  done

  cmd+=(
    --parallel
    --profile
    --output-csv "$OUT_DIR/${query}.csv"
    --output-latex "$OUT_DIR/${query}.tex"
    -q "$query"
  )

  "${cmd[@]}" 2>&1 | tee "$OUT_DIR/${query}.log"
done