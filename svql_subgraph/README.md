# svql_subgraph

## Purpose
Implements the core subgraph isomorphism algorithm

## Key Responsibilities
- **Isomorphism Kernel**: Executes a backtracking search to find bijective mappings between a pattern and a target design.
- **Constraint Validation**: Enforces hardware-specific connectivity rules, ensuring that matched cells in the haystack share the same local topology as the needle.

## Core Abstractions
| Type / Trait | Description |
| :--- | :--- |
| `SubgraphMatcher` | Orchestrates the search process. |
| `GraphIndex` | Lookup tables for cells in the design to provide constant time access for needed attributes. |
| `SingleAssignment` | A partial mapping between of needle cells and haystack cells. |
| `CellWrapper` | A wrapper around netlist cells with access to source metadata. |

## Design Decisions
- **Graph Index**: This was implemented as an optimization to cache results that are needed often such as fan-in/fan-out of cells.
- **Topological Search Order**: The matcher traverses the needle in topological order. This ensures that when a cell is matched, its inputs are often already matched which can be used to help pruning.

## Data Flow
- **Input**: `prjunnamed_netlist::Design` objects for both the pattern and the target as well as search configuration.
- **Output**: `AssignmentSet` containing all valid structural mappings found.

## Usage Example
```rust
use svql_subgraph::{SubgraphMatcher, GraphIndex};
use svql_common::Config;

fn find_matches(needle: &Design, haystack: &Design) {
    let config = Config::default();
    
    let needle_idx = GraphIndex::build(needle);
    let haystack_idx = GraphIndex::build(haystack);
    
    let assignments = SubgraphMatcher::enumerate_with_indices(
        needle,
        haystack,
        &needle_idx,
        &haystack_idx,
        &config,
    );

    println!("Found {} matches", assignments.len());
}
```

## Implementation Notes
- **Performance**: The search is parallelized using `rayon` when the `rayon` feature is enabled.
- **Constraints**: The algorithm assumes the netlist has been flattened and processed by Yosys (e.g., `proc` and `flatten` passes).
