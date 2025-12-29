# SVQL (Structural Verilog Query Language)

## Overview
SVQL is a framework for structural pattern matching in hardware netlists. It provides a domain-specific language (DSL) to define hierarchical queries in terms of composition of hardware and executes them against flattened netlists using a specialized subgraph isomorphism engine.

## System Architecture
The workspace is organized into functional layers that separate query definition from the underlying graph algorithms.

| Layer | Crate | Responsibility |
| :--- | :--- | :--- |
| **DSL** | `svql_macros` | Procedural macros for structural pattern definition (`netlist`, `composite`, `variant`). |
| **Orchestration** | `svql_query` | Type-level state management (`Search` vs `Match`) and query abstractions. |
| **Management** | `svql_driver` | Design ingestion and resource management. |
| **Kernel** | `svql_subgraph` | Subgraph isomorphism implementation. |
| **Infrastructure** | `svql_common` | External tool bindings (Yosys), shared configurations, and test utilities. |

## Execution Flow
1.  **Definition**: Users define queries using `svql_macros` & `svql_query` to compose primitives into complex structures, which are then executed.
2.  **Ingestion**: `svql_driver` invokes Yosys to ingest netlists
4.  **Execution**: `svql_query` orchestrates the search dispatches subgraph queries to `svql_subgraph`, which identifies mappings via a backtracking algorithm.
5.  **Reporting**: Results are bound to the `Match` state which provides source-level traceability.

## Workspace Structure
- `svql_common/`: Yosys management, and shared configuration.
- `svql_driver/`: Thread-safe design management and lifetime management.
- `svql_macros/`: Procedural macro implementations for the DSL attributes.
- `svql_query/`: Query logic, traits, and primitive library.
- `svql_subgraph/`: Backtracking subgraph isomorphism algorithm.
- `prjunnamed/`: External netlist abstraction layer.

## Requirements
- Requires `yosys` in the system `PATH`
- Certain designs (e.g., Hack@DAC21) will require TabbyCAD forverific support.

## Usage Example
```rust
use svql_query::{Search, State, Instance, traits::*};
use svql_driver::Driver;

// 1. Define a hierarchical pattern
#[composite]
pub struct MyQuery<S: State> {
    #[path] path: Instance,
    #[submodule] gate: AndGate<S>,
    #[submodule] reg: Sdffe<S>,
}

impl<S: State> Topology<S> for MyQuery<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // Define internal connectivity constraints
        ctx.connect(Some(&self.gate.y), Some(&self.reg.d));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = Driver::new_workspace()?;
    let config = Config::default();

    // 2. Load the target design (haystack)
    let (key, design) = driver.get_or_load_design(
        "path/to/design.v",
        "top_module",
        &config.haystack_options
    )?;

    // 3. Prepare context and execute
    let context = MyQuery::<Search>::context(&driver, &config.needle_options)?
        .with_design(key.clone(), design);

    let query = MyQuery::<Search>::instantiate(Instance::root("q"));
    let matches = query.query(&driver, &context, &key, &config);

    println!("Found {} matches", matches.len());
    Ok(())
}
```

## Implementation Notes
- **Complexity**: Subgraph isomorphism is NP-complete; SVQL mitigates this using hardware-specific heuristics such as cell types, and fan-in degree.
- **Parallelism**: The matcher supports parallelizing the search across independent candidate branches via the use of `rayon`.
- **Memory**: Large designs require a significant amount of memory to store the matches of a search when many matches are found.