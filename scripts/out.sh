#! /usr/bin/env bash

# set -e

# rm -rf generated
# mkdir -p generated
# cargo expand -p svql_query composites::dff_then_and --lib > generated/svql_query_expanded.rs

uv run scripts/md_tree.py --root . \
    --include 'svql_*/src/**.rs' \
    --include 'README.md' \
    --include 'TODO.md' \
    --include 'Cargo.toml' \
    --include 'svql_*/README.md' \
    --include 'svql_*/Cargo.toml' \
    --exclude 'examples/fixtures/larger_designs/**' \
    --header-base-level 4 \
    > out.md

# uv run scripts/md_tree.py --root . \
#     --include 'svql_macros/src/**.rs' \
#     --include 'svql_query/src/**.rs' \
#     --include 'README.md' \
#     --include 'TODO.md' \
#     --include 'Cargo.toml' \
#     --include 'svql_*/README.md' \
#     --include 'svql_*/Cargo.toml' \
#     --exclude 'examples/fixtures/larger_designs/**' \
#     --header-base-level 4 \
#     > out.md