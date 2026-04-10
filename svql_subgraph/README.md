# svql_subgraph: Subgraph Isomorphism Kernel

**Purpose**  
Implements the bijective pattern-matching algorithm for SVQL. Finds all instances of a "needle" pattern (query netlist) within a larger "haystack" design (target netlist) via subgraph isomorphism. Operates on flattened, gate-level netlists represented as directed graphs where nodes are logic cells and edges are signal connections.

## Core Abstractions

| Type | Description |
|------|-------------|
| **`SubgraphMatcher`** | Orchestrates the search; entry point for enumeration. |
| **`SubgraphMatcherCore`** | Internal state holding references to designs, indices, and atomic counters for parallel execution. |
| **`GraphIndex`** | Pre-computed lookup tables (topological order, fan-in/out sets, cell-type buckets) enabling O(1) graph traversal. |
| **`SingleAssignment`** | Partial bijective mapping $M: V_{needle} \rightarrow V_{haystack}$ tracking which pattern cells map to design cells. |
| **`AssignmentSet`** | Collection of all valid complete mappings found during search. |
| **`CellWrapper`** | Typed cell reference wrapping `prjunnamed_netlist::CellRef` with metadata access (kind, debug ID, source location). |

## Algorithm

### 1. Indexing
Builds `GraphIndex` for both needle and haystack:
- **Topological Sort**: Cells ordered from inputs → gates → outputs (DFFs treated as inputs for ordering purposes).
- **Connectivity Maps**: Fan-in and fan-out adjacency lists with port indices.
- **Type Buckets**: Cells grouped by `CellKind` for rapid candidate filtering.

### 2. Backtracking Search
Traverses needle cells in **reverse topological order** (ensuring inputs are mapped before gates, gates before outputs):
- For current unmapped needle cell $u$, generate candidate set $C(u) \subseteq V_{haystack}$.
- **Candidate Criteria**:
  1. **Unmapped**: $v$ not already assigned in current `SingleAssignment`.
  2. **Type Compatible**: `CellKind` of $u$ matches $v$.
  3. **Fan-in Consistency**: For all already-mapped predecessors $u'$ of $u$, their images $v'$ must drive $v$ on corresponding ports.
  4. **Fan-out Intersection**: $v$ must be in the fan-out of all mapped predecessors (computed via set intersection for efficiency).

### 3. Constraint Validation
For each candidate $v$:
- Verifies port connectivity matches exactly (handles commutative inputs for AND/OR/XOR via permutation checks).
- Validates DFF control signals (clock, reset, enable) match between needle and haystack cells.

### 4. Recursion & Backtracking
- If $C(u)$ empty: backtrack.
- If candidates exist: recurse on each with updated assignment.
- When all cells mapped: record assignment in `AssignmentSet`.

### 5. Deduplication
Filters duplicate matches using signatures of internal logic cells (ignoring I/O ports). Uses bitmask to exclude Input/Output cells from signature if internal cells exist; otherwise uses full signature.

## Key Optimizations

**Topological Ordering**  
Processing gates before outputs ensures fan-in constraints are checkable early, enabling aggressive pruning of invalid branches.

**Set Intersection Candidate Generation**  
Instead of scanning all haystack cells, computes candidates as intersection of fan-out sets from already-mapped predecessors:
```rust
candidates = fanout(mapped_pred[0]) ∩ fanout(mapped_pred[1]) ∩ ...
```

**Parallel Root Search**  
Uses `rayon` to parallelize the outer loop over root candidates (initial matches for the first needle cell). Atomic counters track:
- `branches_explored`: Total search tree nodes visited.
- `active_branches`: Currently active parallel workers.
- `matches_found`: Total valid matches discovered.
- `initial_candidates_done/total`: Progress tracking for root candidates.

**Single-Cell Heuristic**  
Patterns consisting of a single cell type bypass the full backtracking algorithm and use direct filtering of the `GraphIndex` cell-type bucket.

**Match-Length Strategies**  
Configurable via `Config::match_length`:
- `First`: Stop after first match (fastest).
- `NeedleSubsetHaystack`: Pattern must be subgraph of design (allows extra design nodes).
- `Exact`: Bijective isomorphism only (strictest).

## Integration with Ecosystem

- **Caller**: Invoked by `svql_query` during `Netlist` pattern execution via `SubgraphMatcher::enumerate_with_indices()`.
- **Input**: Receives `prjunnamed_netlist::Design` objects and pre-built `GraphIndex` instances from `svql_driver`.
- **Output**: Returns `AssignmentSet` containing bijective mappings (`HashMap<GraphNodeIdx, GraphNodeIdx>`), which `svql_query` converts into columnar `Table` rows.
- **Performance**: Designed as the inner loop of the query system; optimized for speed over memory (indices are pre-computed).

## Usage Example

```rust
use svql_subgraph::SubgraphMatcher;
use svql_common::{Config, GraphIndex};

// Designs loaded via svql_driver
let needle = &design_container.design();
let haystack = &target_container.design();
let needle_idx = design_container.index();
let haystack_idx = target_container.index();

let config = Config::builder()
    .match_length(MatchLength::NeedleSubsetHaystack)
    .parallel(true)
    .build();

let assignments = SubgraphMatcher::enumerate_with_indices(
    needle,
    haystack,
    needle_idx,
    haystack_idx,
    "needle_module".to_string(),
    "haystack_module".to_string(),
    &config,
);

println!("Found {} matches", assignments.len());
for assignment in &assignments.items {
    for (needle_node, haystack_node) in assignment.needle_mapping() {
        // Process bijective mapping
    }
}
```