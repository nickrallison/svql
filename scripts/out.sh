#! /usr/bin/env bash

# set -e

rm -rf generated
mkdir -p generated

uv run scripts/md_tree.py --root . \
    --include 'svql_macros/**.rs' \
    --include 'svql_query/**.rs' \
    --include 'examples/**.v' \
    --include 'examples/**.il' \
    --include 'Cargo.toml' \
    --include '**/Cargo.toml' \
    --exclude 'examples/fixtures/larger_designs/**' \
    --header-base-level 2 \
    > out.md

# uv run scripts/md_tree.py --root . \
#     --include 'svql_macros/**.rs' \
#     --include 'svql_query/**.rs' \
#     --exclude 'examples/fixtures/larger_designs/**' \
#     --header-base-level 2 \
#     > out.md

