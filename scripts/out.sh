#! /usr/bin/env bash

# md_tree.py --root . \
#     --include 'svql_*/**.rs' \
#     --include 'examples/**.v' \
#     --include 'Cargo.toml' \
#     --include 'README.md' \
#     --include 'svql_*/Cargo.toml' \
#     --header-base-level 2 \
#     --section "examples=Examples" \
#     --section "svql_subgraph=svql_subgraph" \
#     --section "svql_driver=svql_driver" \
#     --section "svql_query=svql_query" \
#     > out.md

md_tree.py --root . \
    --include 'svql_subgraph/**.rs' \
    --include 'examples/**.v' \
    --include 'Cargo.toml' \
    --include 'README.md' \
    --include 'svql_subgraph/Cargo.toml' \
    --header-base-level 2 \
    --section "examples=Examples" \
    --section "svql_subgraph=svql_subgraph" \
    > out.md