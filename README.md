# SVQL (SystemVerilog Query Language)

## Overview
SVQL is a framework for structural pattern matching in hardware netlists. It uses a Rust-based DSL to define hierarchical queries executed against flattened netlists using a columnar storage model and join optimizations.

## implementation Model
SVQL uses a DataFrame-style architecture:
- **Patterns**: Rust structs using derive macros.
- **Store**: Central repository for all session results.
- **Tables**: Columnar storage for specific pattern types.
- **Rows**: Snapshots of individual matches.

## Query Library
SVQL provides patterns for detecting Common Weakness Enumerations (CWEs).

### CWE-1234: Internal or Debug Modes Allow Override of Locks
Matches faulty unlock logic feeding a locked register.

```rust
#[derive(Composite)]
#[connection(from = ["unlock_logic", "unlock"], to = ["locked_register", "write_en"])]
pub struct Cwe1234 {
    #[submodule]
    pub unlock_logic: UnlockLogic,
    #[submodule]
    pub locked_register: LockedRegister,
}
```

## System Architecture

| Layer | Crate | Responsibility |
| :--- | :--- | :--- |
| **DSL** | `svql_macros` | Proc-macros for `Netlist`, `Composite`, and `Variant` patterns. |
| **Session** | `svql_query` | Columnar storage, Execution DAG, and Join Planner. |
| **Management** | `svql_driver` | Design ingestion, caching, and Graph Indexing. |
| **Kernel** | `svql_subgraph` | Bijective subgraph isomorphism matching. |

## Usage Example
```rust
use svql_query::prelude::*;
use svql_query_lib::security::cwe1234::Cwe1234;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = Driver::new_workspace()?;
    let config = Config::default();
    let key = DriverKey::new("design.json", "top_module");

    // Execute query and resolve dependencies
    let store = svql_query::run_query::<Cwe1234>(&driver, &key, &config)?;

    if let Some(table) = store.get::<Cwe1234>() {
        for row in table.rows() {
            println!("{}", row.render(&store, &driver, &key));
        }
    }
    Ok(())
}
```
