#! /usr/bin/env bash

# set -e

rm -rf generated
mkdir -p generated
cargo expand --lib --package svql_query queries > generated/expanded.rs

# /home/nick/Projects/svql/.venv/bin/python3 scripts/md_tree.py --root . \
#     --include 'svql_query/**.rs' \
#     --include 'svql_driver/**.rs' \
#     --include 'svql_subgraph/**.rs' \
#     --include 'svql_common/**.rs' \
#     --include 'generated/**.rs' \
#     --include 'Cargo.toml' \
#     --header-base-level 2 \
#     --section "examples=Examples" \
#     --section "generated=Generated" \
#     --section "svql_subgraph=svql_subgraph" \
#     --section "svql_driver=svql_driver" \
#     --section "svql_query=svql_query" \
#     --section "svql_common=svql_common" \
#     > out.md

uv run scripts/md_tree.py --root . \
    --include 'svql_query/**.rs' \
    --include 'examples/**.v' \
    --include 'generated/**.rs' \
    --include 'Cargo.toml' \
    --exclude 'examples/fixtures/larger_designs/**' \
    --header-base-level 2 \
    > out.md
