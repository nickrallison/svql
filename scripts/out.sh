#! /usr/bin/env bash

# set -e

# rm -rf generated
# mkdir -p generated
# cargo expand -p svql_query composites::dff_then_and --lib > generated/svql_query_expanded.rs

uv run scripts/md_tree.py --root . \
    --include 'svql_subgraph/src/**.rs' \
    --include 'svql_query/src/**.rs' \
    --include 'prjunnamed/netlist/**.rs' \
    --include 'Cargo.toml' \
    --include 'svql_subgraph/Cargo.toml' \
    --include 'svql_query/Cargo.toml' \
    --exclude 'examples/fixtures/larger_designs/**' \
    --header-base-level 4 \
    > out.md

# uv run scripts/md_tree.py --root . \
#     --include 'svql_query/src/**.rs' \
#     --include 'Cargo.toml' \
#     --include 'svql_subgraph/Cargo.toml' \
#     --include 'svql_query/Cargo.toml' \
#     --exclude 'examples/fixtures/larger_designs/**' \
#     --header-base-level 4 \
#     > out.md