#! /usr/bin/env bash

mkdir -p generated
# cargo expand --package svql_query queries::netlist > generated/expanded_netlist.rs

# python3 scripts/md_tree.py --root . \
#     --include 'svql_*/**.rs' \
#     --include 'examples/**.v' \
#     --include 'generated/**.rs' \
#     --include 'Cargo.toml' \
#     --include 'README.md' \
#     --include 'svql_*/Cargo.toml' \
#     --include 'svql_*/README.md' \
#     --header-base-level 2 \
#     --section "examples=Examples" \
#     --section "generated=Generated" \
#     --section "svql_subgraph=svql_subgraph" \
#     --section "svql_driver=svql_driver" \
#     --section "svql_query=svql_query" \
#     > out.md

# mkdir -p generated
# cargo expand --package svql_query queries::netlist > generated/expanded_netlist.rs

python3 scripts/md_tree.py --root . \
    --include 'svql_subgraph/**.rs' \
    --include 'examples/**.v' \
    --include 'Cargo.toml' \
    --include 'README.md' \
    --include 'svql_subgraph/Cargo.toml' \
    --include 'svql_subgraph/README.md' \
    --header-base-level 2 \
    --section "examples=Examples" \
    --section "generated=Generated" \
    --section "svql_subgraph=svql_subgraph" \
    --section "svql_driver=svql_driver" \
    --section "svql_query=svql_query" \
    > out.md
