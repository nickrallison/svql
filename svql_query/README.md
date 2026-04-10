# svql_query: Execution Engine & Columnar Storage

**Purpose**  
Core orchestration layer for SVQL pattern matching. Manages query execution workflows using a DataFrame-style architecture with columnar storage, enabling type-safe, composable queries that detect security vulnerabilities in flattened gate-level netlists.

## Core Architecture

### Columnar Storage Model
- **`Table<T>`**: Type-safe columnar store where each row represents one pattern match. Backed by `ColumnStore` (HashMap of column names to `Vec<ColumnEntry>`).
- **`ColumnEntry`**: Union type storing `Wire` (cell references), `Ref<T>` (typed row pointers to other tables), `WireArray` (bundles), or `MetaValue` (discriminants, counts, cell IDs).
- **`Store`**: Type-erased registry (`HashMap<TypeId, Arc<dyn AnyTable + Send + Sync>>`) enabling cross-pattern references (e.g., Composite → Submodule).
- **`Row<T>`**: Typed view into a table row providing `wire()`, `sub()`, `meta()`, and `resolve()` (hierarchical path traversal) methods.

### Pattern Type System
Five algebraic pattern kinds implemented via `Pattern` and `Component` traits:

1. **Netlist**: Atomic patterns matching external Verilog/RTLIL modules via subgraph isomorphism (delegated to `svql_subgraph`).
2. **Primitive**: Direct cell-type matches (e.g., AND gates) using `GraphIndex` cell-type buckets; bypasses expensive subgraph search.
3. **Composite**: Hierarchical compositions of sub-patterns with connectivity constraints. Uses incremental joins rather than full Cartesian products.
4. **Variant**: Polymorphic unions (sum types) where results are the union of constituent sub-queries.
5. **Recursive**: Self-referential tree patterns (e.g., logic cones, AND trees) built via fixpoint iteration over base pattern matches.

### State Tracking
Uses phantom types to distinguish between `Search` (unbound pattern definition) and `Match` (bound results with resolved `Wire` references) phases, preventing accidental use of unbound variables in logic expecting design-matched cells.

## Execution Flow

### 1. Plan Construction (`ExecutionPlan::build`)
- Parses the pattern's dependency graph from `Pattern::EXEC_INFO` (which contains `type_id`, `type_name`, and `nested_dependancies`).
- Constructs a DAG where nodes are sub-queries and edges are data dependencies.
- Assigns each node a `TableSlot` (synchronization primitive).

### 2. Parallel Dispatch
- Uses `rayon` for work-stealing parallelism.
- **`TableSlot`**: Hybrid `OnceLock` + `Condvar` ensuring each pattern is computed exactly once:
  - `ClaimResult::Ready`: Already computed (lock-free read).
  - `ClaimResult::Claimed`: Current thread won the race; must execute and fill.
  - `ClaimResult::Wait`: Another thread is computing; block on condvar.

### 3. Search Execution
- **Netlist**: Calls `svql_subgraph::SubgraphMatcher::enumerate_with_indices()`.
- **Primitive**: Filters `GraphIndex` by `CellKind` and optional custom filter (e.g., DFF with specific control signals).
- **Composite**: 
  - Builds bipartite connectivity indices (`ConnectivityCache`) mapping valid connections between sub-query results.
  - Uses greedy join ordering (smallest table first).
  - Incrementally joins tables, filtering by connection constraints at each step.
  - Applies user-defined filters and resolves aliases.
- **Variant**: Executes all arms in parallel (if configured), concatenates results, and maps ports via `PortMap` definitions.

### 4. Storage & Rehydration
- Results populate `Store` via `store.insert<T>(table)`.
- `Pattern::rehydrate(row, store, driver, key, config)` reconstructs the original Rust struct from columnar data for programmatic analysis.

## Key Abstractions

| Type | Role |
|------|------|
| **`Pattern`** | Core trait defining schema, dependencies (`EXEC_INFO`), search logic, and rehydration. |
| **`ExecutionContext`** | Provides access to `Driver`, design key, config, and completed dependency tables (`get_any_table`). |
| **`ExecInfo`** | Static metadata for DAG construction: `type_id`, `type_name`, `search_function` pointer, and `nested_dependancies`. |
| **`Ref<T>`** | Opaque, type-safe row index (`RowIndex` + PhantomData). Prevents invalid cross-table references. |
| **`ConnectivityCache`** | Pre-computed bipartite indices for Composite joins, mapping valid connections between sub-query tables. |

## Integration with Ecosystem

- **Input**: Accepts `Driver` and `DriverKey` from `svql_driver` to load designs.
- **Matching**: Delegates bijective pattern matching to `svql_subgraph` for Netlist patterns.
- **Output**: Returns `Store` containing all result tables, or renders hierarchical `ReportNode` trees via `row_to_report_node()` for display.
- **Entry Point**: `run_query::<P>(driver, key, config)` executes pattern `P` and returns populated `Store`.

## Performance Features

- **Parallel Execution**: Work-stealing via `rayon` across independent sub-queries.
- **Connectivity Indexing**: For Composites, builds bipartite indices to avoid $O(n^k)$ Cartesian products.
- **Single-Cell Heuristic**: Primitives bypass subgraph isomorphism entirely.
- **Automatic Deduplication**: Uses `EntrySignature` (hash of wire IDs and submodule refs) to filter duplicate results.
- **Lazy Evaluation**: Results computed on-demand per `TableSlot` claims.

## Usage Example

```rust
use svql_query::prelude::*;
use svql_query_lib::security::cwe1234::Cwe1234;

let driver = Driver::new_workspace()?;
let key = DriverKey::new("design.json", "top");
let config = Config::builder().parallel(true).build();

// Execute query
let store = run_query::<Cwe1234>(&driver, &key, &config)?;

// Access results
if let Some(table) = store.get::<Cwe1234>() {
    for (ref, row) in table.rows() {
        let instance = Cwe1234::rehydrate(&row, &store, &driver, &key, &config)?;
        println!("Found match at: {:?}", instance.dff_enable.write_en());
    }
}
```