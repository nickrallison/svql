#!/usr/bin/env bash
set -euo pipefail

# Resolve run-files ----------------------------------------------------------
yosys_bin=$(rlocation yosys_src/yosys_build/bin/yosys)
plugin=$(rlocation svql_driver/svql.so)
variant=$(rlocation svql_driver/examples/cwe1234/variant1.v)
pattern=$(rlocation svql_driver/examples/cwe1234/locked_register_pat.v)

# Run exactly the same Yosys flow as before ----------------------------------
"$yosys_bin" \
  -m "$plugin" \
  "$variant" \
  -p "hierarchy -top locked_register_example" \
  -p "proc" \
  -p "svql -map $pattern -verbose"