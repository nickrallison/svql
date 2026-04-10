# svql_driver: Design Lifecycle & Caching Layer

**Purpose**  
Manages the ingestion, synthesis, and caching of hardware designs. Provides a thread-safe registry that ensures expensive Yosys synthesis and graph indexing operations execute only once per unique `(file_path, module_name)` pair.

## Core Abstractions

| Type | Description |
|------|-------------|
| **`Driver`** | Central coordinator holding the workspace root, Yosys binary path, and design registry. Primary API for loading designs. |
| **`DriverKey`** | Composite key `(PathBuf, String)` uniquely identifying a design instance. Used for cache lookups. |
| **`DesignContainer`** | Self-referential struct (via `ouroboros`) pairing a `prjunnamed_netlist::Design` with its pre-computed `GraphIndex`. Guarantees index validity for the design's lifetime. |
| **`ModuleConfig`** | Configuration for Yosys processing (flatten, opt_clean, verific, raw JSON load). |

## Architecture

**Thread-Safe Caching Registry**  
Uses `Arc<RwLock<HashMap<DriverKey, Arc<DesignContainer>>>>` to enable concurrent design loading across threads. Designs are cached indefinitely; subsequent requests return the cached `Arc<DesignContainer>` without re-synthesis.

**Self-Referential Storage**  
`DesignContainer` employs `ouroboros` to create a self-referencing struct where `GraphIndex` (holding references to the design) remains valid as long as the container exists. This eliminates complex lifetime management while ensuring consistency between the netlist and its indices.

**Workspace-Centric Resolution**  
Relative paths are resolved against the Cargo workspace root. Supports both workspace-relative and absolute paths.

## Data Flow

```
Input: (file_path, module_name, ModuleConfig)
    ↓
Driver::get_design()
    ↓
[Cache Hit] → Return Arc<DesignContainer>
[Cache Miss] ↓
YosysModule::import_design()
    ↓
Yosys Pipeline (read_verilog → proc → flatten → opt → write_json)
    ↓
prjunnamed_netlist::Design (parsed from JSON)
    ↓
GraphIndex::build() (topological sort + connectivity maps)
    ↓
DesignContainer { design, index }
    ↓
Registry Insertion
    ↓
Output: Arc<DesignContainer>
```

## Key Implementation Details

**Yosys Integration**  
- Invokes external Yosys binary via configurable pipeline steps defined in `ModuleConfig`.
- Supports raw JSON import (`load_raw`) to skip synthesis for pre-processed netlists.
- Automatically categorizes input files by extension (`.v`, `.il`, `.json`) to determine read command.

**Lazy Loading & Preloading**  
- `get_design()` implements lazy loading: checks registry first, synthesizes on miss.
- `preload_design()` allows eager cache warming before query execution to avoid latency during pattern matching.

**Source Location Tracking**  
Provides reverse lookup from `PhysicalCellId` to source code locations via the stored `GraphIndex`, enabling query results to report original Verilog line numbers.

**Error Handling**  
Returns `DriverError` for Yosys binary not found, I/O failures, or design loading errors. Panics only on internal registry lock poisoning.

## Integration with Ecosystem

- **Consumer**: `svql_query` calls `Driver::get_design()` to obtain `DesignContainer` instances.
- **Output**: The `GraphIndex` built by the driver is the primary input to `svql_subgraph` for pattern matching.
- **Lifetime**: Design containers persist for the duration of the query session; source locations accessed via `Driver::get_cell_source()`.

## Usage Example

```rust
let driver = Driver::new_workspace()?;
let key = DriverKey::new("design.json", "top_module");

// First call: loads, indexes, caches
let container = driver.get_design(&key, &config)?;

// Subsequent calls: O(1) cache hit
let container = driver.get_design(&key, &config)?;
let index = container.index(); // GraphIndex for subgraph matching
```