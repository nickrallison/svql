#!/usr/bin/env bash
set -euo pipefail

sanitize() {
  echo "$1" \
    | sed 's# --module #_#g' \
    | sed 's# --raw##g' \
    | sed 's#[/ ]#_#g' \
    | sed 's#[^A-Za-z0-9._-]#_#g'
}


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
  for design in "${DESIGNS[@]}"; do
    design_tag="$(sanitize "$design")"

    echo "============================================================"
    echo "Running query: $query"
    echo "Design: $design"
    echo "============================================================"

    cargo run \
      --profile release \
      --package svql_cli \
      --bin svql_cli \
      -- \
      --design "$design" \
      --parallel \
      --profile \
      --output-csv "$OUT_DIR/${design_tag}_${query}.csv" \
      --output-latex "$OUT_DIR/${design_tag}_${query}.tex" \
      -q "$query" \
      2>&1 | tee "$OUT_DIR/${design_tag}_${query}.log"
  done
done