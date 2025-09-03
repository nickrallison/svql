#! /usr/bin/env bash

# set -e

# mkdir -p generated
# cargo expand --lib --package svql_query queries::netlist > generated/expanded_netlist.rs
# cargo build

# # Find and copy the most recent version of each generated test file
# echo "Finding most recent generated test files..."

# # For svql_subgraph
# echo "Finding svql_subgraph_generated_tests.rs"
# find target -name "svql_subgraph_generated_tests.rs" -printf "%T+ %p\n" | sort -r | head -n 1 | awk '{print $2}' | xargs -I{} cp {} generated/svql_subgraph_generated_tests.rs

# # For svql_driver
# echo "Finding svql_driver_generated_tests.rs"
# find target -name "svql_driver_generated_tests.rs" -printf "%T+ %p\n" | sort -r | head -n 1 | awk '{print $2}' | xargs -I{} cp {} generated/svql_driver_generated_tests.rs

# # For svql_query
# echo "Finding svql_query_generated_tests.rs"
# find target -name "svql_query_generated_tests.rs" -printf "%T+ %p\n" | sort -r | head -n 1 | awk '{print $2}' | xargs -I{} cp {} generated/svql_query_generated_tests.rs


# python3 scripts/md_tree.py --root . \
#   --include 'prjunnamed/netlist/src/design.rs' \
#   --include 'prjunnamed/netlist/src/cell.rs' \
#   --include 'svql_*/**.rs' \
#   --include 'examples/**.v' \
#   --exclude 'examples/fixtures/larger_designs/verilog/openpiton_system.v' \
#   --include 'generated/**.rs' \
#   --include 'Cargo.toml' \
#   --include 'README.md' \
#   --include 'svql_*/Cargo.toml' \
#   --include 'svql_*/README.md' \
#   --header-base-level 2 \
#   --section "examples=Examples" \
#   --section "generated=Generated" \
#   --section "svql_subgraph=svql_subgraph" \
#   --section "svql_driver=svql_driver" \
#   --section "svql_query=svql_query" \
#   > out.md

#############################################

mkdir -p generated
# cargo expand --lib --package svql_query queries::netlist > generated/expanded_netlist.rs
# cargo build

# Find and copy the most recent version of each generated test file
echo "Finding most recent generated test files..."

# For svql_subgraph
echo "Finding svql_subgraph_generated_tests.rs"
find target -name "svql_subgraph_generated_tests.rs" -printf "%T+ %p\n" | sort -r | head -n 1 | awk '{print $2}' | xargs -I{} cp {} generated/svql_subgraph_generated_tests.rs

# # For svql_driver
# echo "Finding svql_driver_generated_tests.rs"
# find target -name "svql_driver_generated_tests.rs" -printf "%T+ %p\n" | sort -r | head -n 1 | awk '{print $2}' | xargs -I{} cp {} generated/svql_driver_generated_tests.rs

# # For svql_query
# echo "Finding svql_query_generated_tests.rs"
# find target -name "svql_query_generated_tests.rs" -printf "%T+ %p\n" | sort -r | head -n 1 | awk '{print $2}' | xargs -I{} cp {} generated/svql_query_generated_tests.rs


# python3 scripts/md_tree.py --root . \
#     --include 'prjunnamed/netlist/src/design.rs' \
#     --include 'prjunnamed/netlist/src/cell.rs' \
#     --include 'prjunnamed/netlist/src/cell/flip_flop.rs' \
#     --include 'prjunnamed/netlist/src/value.rs' \
#     --include 'svql_subgraph/**.rs' \
#     --include 'examples/**.v' \
#     --include 'examples/patterns/security/access_control/locked_reg/rtlil/*.il' \
#     --include 'examples/fixtures/security/access_control/locked_reg/rtlil/*.il' \
#     --exclude 'examples/fixtures/larger_designs/verilog/openpiton_system.v' \
#     --include 'generated/**.rs' \
#     --include 'Cargo.toml' \
#     --include 'README.md' \
#     --include 'svql_subgraph/Cargo.toml' \
#     --include 'svql_subgraph/README.md' \
#     --header-base-level 2 \
#     --section "examples=Examples" \
#     --section "generated=Generated" \
#     --section "svql_subgraph=svql_subgraph" \
#     --section "svql_driver=svql_driver" \
#     --section "svql_query=svql_query" \
#     > out.md

python3 scripts/md_tree.py --root . \
    --include 'svql_subgraph/src/lib.rs' \
    --include 'svql_common/**.rs' \
    --include 'Cargo.toml' \
    --header-base-level 2 \
    --section "examples=Examples" \
    --section "generated=Generated" \
    --section "svql_subgraph=svql_subgraph" \
    --section "svql_driver=svql_driver" \
    --section "svql_query=svql_query" \
    --section "svql_common=svql_common" \
    > out.md
