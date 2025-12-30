# svql_driver

## Purpose
Manages the lifetime of hardware designs, providing a thread-safe registry for loading, caching, and indexing netlists to prevent redundant processing during query execution.

## Key Responsibilities
- **Design Ingestion**: Orchestrates the transformation of netlists files via Yosys.
- **Resource Caching**: Maintains a registry of loaded designs to ensure that each unique module is available during the search process.

## Core Abstractions
| Type / Trait | Description |
| :--- | :--- |
| `Driver` | The central coordinator for design loading and access. |
| `DriverKey` | A unique identifier for a design. |
| `DesignContainer` | A container that pairs a `Design` with its associated `GraphIndex`. |
| `Context` | A collection of designs used as the target search space for a query operation. |

## Data Flow
- **Input**: Filesystem paths to HDL source, top-level module names, and `ModuleConfig` parameters.
- **Output**: `DesignContainer` instances and `Context` objects to manage the netlists.

## Usage Example
```rust
use svql_driver::Driver;
use svql_common::ModuleConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = Driver::new_workspace()?;
    let config = ModuleConfig::default().with_flatten(true);

    // Load and index a design
    let (key, design) = driver.get_or_load_design(
        "path/to/design.v",
        "top_module",
        &config
    )?;

    // Create a context for query execution
    let context = driver.create_context_single(&key)?;
    
    assert!(context.contains(&key));
    Ok(())
}
```

## Implementation Notes
- **Thread Safety**: The internal registry uses `Arc<RwLock<HashMap<...>>>` to allow concurrent design submission & retrieval & submission across multiple threads.
- **Memory Management**: Uses `ouroboros` to implement a self-referencing `DesignContainer`.