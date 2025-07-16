#!/usr/bin/env bash
set -euo pipefail

./yosys/yosys -m ./build/svql_driver/libsvql_driver.so -p "read_verilog examples/cwe1234/locked_register_pat.v" -p "hierarchy -top locked_register_example" -p "proc" -p "svql_driver -pat svql_query/verilog/and.v and_gate -verbose"