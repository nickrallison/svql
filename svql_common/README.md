# svql_common: Shared Types & Hardware Abstractions

**Purpose**  
Foundational crate providing core data types, hardware abstractions, and Yosys integration used across the entire SVQL ecosystem. Defines the wire/cell representation, graph indexing primitives, configuration structures, and the design ingestion pipeline.

## Core Abstractions

| Type | Description |
|------|-------------|
| **`Wire`** | Bundle of one or more `Net` references representing a hardware signal. Supports slicing, concatenation, and constant detection. |
| **`CellKind`** | Enumeration of hardware primitives (And, Or, Dff, Mux, etc.) with predicates (`is_logic_gate()`, `has_commutative_inputs()`). |
| **`CellWrapper`** | Safe wrapper around `prjunnamed_netlist::CellRef` providing typed access to cell data, metadata, and source locations. |
| **`PhysicalCellId`** | Stable identifier from netlist source (debug_index) used for persistent storage and cross-referencing between queries. |
| **`GraphNodeIdx`** | Local index within a specific `GraphIndex` array; used exclusively by the subgraph solver for performance. |
| **`GraphIndex`** | Pre-computed structural index of a design containing topological ordering, fan-in/out maps, and cell-type buckets. |
| **`YosysModule`** | Handle to a specific module within a source file; manages Yosys invocation and transformation pipeline. |
| **`ModuleConfig`** | Configuration for Yosys processing (flatten, opt_clean, verific, parameters, custom steps). |
| **`Config`** | Query execution configuration including match length strategy, parallelism, and pattern variable matching. |

## Architecture

**Type Hierarchy**  
```
Hardware Primitives
├── CellKind (enum: And, Or, Dff, etc.)
├── CellWrapper (reference + metadata)
└── PhysicalCellId (stable ID)

Connectivity & Indexing
├── GraphNodeIdx (local solver index)
├── GraphIndex (complete design index)
│   ├── CellRegistry (topological ordering)
│   ├── ConnectivityGraph (fan-in/out maps)
│   └── IoMapping (port to internal cell mapping)
└── Wire (signal bundle with net references)

Configuration & I/O
├── DesignPath (file type categorization: .v/.il/.json)
├── YosysModule (module handle + synthesis pipeline)
├── ModuleConfig (Yosys-specific options)
└── Config (query execution parameters)
```

**Indexing Pipeline**  
```
Design (prjunnamed_netlist)
    ↓
GraphIndex::build()
    ↓
CellRegistry (filter Name cells, topo sort)
    ↓
ConnectivityGraph (build fan-in/out adjacency)
    ↓
IoMapping (map Input/Output ports to internal cells)
    ↓
GraphIndex (ready for subgraph matching)
```

## Key Implementation Details

**Wire Representation**  
- Wraps `prjunnamed_netlist::Value` (vector of nets).
- Provides `cell_id()` to extract the driving cell for single-net wires.
- Implements `drives()` for connectivity checking between wires.

**Cell Identification**  
- `PhysicalCellId` derived from Yosys debug indices; stable across sessions.
- `GraphNodeIdx` is local to a specific `GraphIndex` instance and used only within the subgraph solver for array indexing.

**Graph Index Structure**  
- **CellRegistry**: Maintains `cells_topo` (topologically sorted cells excluding Name nodes), `cell_id_map` (PhysicalCellId → GraphNodeIdx), and `cell_type_indices` (CellKind → [GraphNodeIdx]).
- **ConnectivityGraph**: Bidirectional adjacency maps with port indices, pre-computed fan-out sets for O(1) intersection operations, and cached "intersection of fanout of fanin" for advanced pruning.
- **IoMapping**: Maps primary input names to their fan-out cells and output names to their fan-in cells.

**Yosys Integration**  
- `DesignPath` categorizes files by extension to select appropriate read command (`read_verilog`, `read_rtlil`, `read_json`).
- `YosysModule::import_design()` generates Yosys command sequences (hierarchy, proc, flatten, opt, write_json) and executes via `std::process::Command`.
- Supports raw JSON import bypassing Yosys for pre-synthesized netlists.

**Configuration**  
- `MatchLength` enum: `First` (early exit), `NeedleSubsetHaystack` (subgraph), `Exact` (isomorphism).
- `ConfigBuilder` fluent API for setting parallelism, match strategies, and Yosys options for needle vs. haystack processing.

## Integration with Ecosystem

- **Foundation**: All crates (`svql_driver`, `svql_query`, `svql_subgraph`, `svql_cli`) depend on this crate for basic types.
- **Bridge**: `YosysModule` and `ModuleConfig` provide the only interface to external synthesis tools.
- **Storage**: `PhysicalCellId` and `Wire` are the primary types stored in query result tables (`svql_query`).
- **Algorithm Input**: `GraphIndex` is the required input for `svql_subgraph` matching algorithms.

## Usage Example

```rust
use svql_common::{YosysModule, ModuleConfig, GraphIndex, Config, MatchLength};

// 1. Load and synthesize a design
let module = YosysModule::new("design.v", "top_module")?;
let config = ModuleConfig::new()
    .with_flatten(true)
    .with_opt_clean(true);
let design = module.import_design(&config)?;

// 2. Build structural index
let index = GraphIndex::build(&design);

// 3. Query graph topology
let cell = index.get_cell_by_index(GraphNodeIdx::new(42));
println!("Cell type: {:?}", cell.cell_type());
println!("Fan-out count: {}", index.fanout_set(GraphNodeIdx::new(42)).len());

// 4. Configure query execution
let query_config = Config::builder()
    .match_length(MatchLength::NeedleSubsetHaystack)
    .parallel(true)
    .build();
```