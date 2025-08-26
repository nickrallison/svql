# svql_subgraph

## Purpose

The svql_subgraph crate implements the core subgraph isomorphism algorithm for hardware pattern matching. It provides the low-level machinery for finding pattern instances within larger designs by solving the subgraph isomorphism problem on hardware netlists. The implementation is optimized for hardware designs which are typically sparse and have directed connections with non-interchangeable ports.

Key capabilities:
- Efficient subgraph matching using heuristic-based anchor selection
- Configurable matching modes (exact vs superset pin matching)
- Multiple deduplication strategies for handling automorphisms
- Detailed binding information between pattern and design elements

This crate forms the foundation that other svql components build upon for hardware analysis tasks.

## Note

It may be worth noting that this implementation was created vs. using the [Ullman Subgraph Isomorphism Algorithm](https://adriann.github.io/Ullman%20subgraph%20isomorphism.html), which when tested performed significantly worse, the reason is not yet clear, but it was tested using the extract pass of Yosys on a submodule of OpenTitan (otbn) which took about 12 minutes to run a query with a modified version of the extract pass.

## Parallelism

svql_subgraph can optionally parallelize the top-level branching of the subgraph search using Rayon.

- Enable with the crate feature `rayon`:
  - In your workspace or command line: `cargo test -p svql_subgraph --features svql_subgraph/rayon`
  - Or within this crate: `cargo test --features rayon`

The parallelization strategy only splits the first branching step, then continues the deeper recursion sequentially. This approach typically delivers strong performance gains without excessive threading overhead.

To control the number of threads Rayon uses, set the environment variable RAYON_NUM_THREADS, for example:
- `RAYON_NUM_THREADS=8 cargo test --features rayon`