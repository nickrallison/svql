# svql_query

## Purpose
Provides the type system and orchestration logic for queries.

## Key Responsibilities
- **Type-Level State Tracking**: Uses the `State` trait to differentiate between `Search` (unbound pattern definition) and `Match` (results bound to design cells) phases.
- **Structural Composition**: Implements the logic for `Netlist` (atomic primitives), `Composite` (hierarchical structures), and `Variant` (polymorphic choices) query types.
- **Execution Orchestration**: Manages the dispatch of queries to the `svql_subgraph` kernel.

## Core Abstractions
| Type / Trait | Description |
| :--- | :--- |
| `State` | Trait defining the phase of a query, implemented by `Search` and `Match`. |
| `Wire<S>` | A logical connection point that holds a `CellWrapper` when in the `Match` state. |
| `Query` | The primary interface for executing structural searches against a design context. |
| `Instance` | A hierarchical path tracker used to identify components within a nested query tree. |

## Data Flow
- **Input**: `Search` state query structures, design handles from `svql_driver`, and search `Config`.
- **Output**: Collections of `Match` state structures and `ReportNode` trees for source-level traceability.

## Usage Example
```rust
use svql_query::{Search, Instance, traits::*};
use svql_driver::Driver;
use svql_common::Config;

fn execute_query<Q>(driver: &Driver, config: &Config, key: &DriverKey) -> Vec<Q::Matched<'_>> 
where 
    Q: Query + Searchable 
{
    let query = Q::instantiate(Instance::root("root".to_string()));
    let context = Q::context(driver, &config.needle_options).unwrap();
    
    query.query(driver, &context, key, config)
}
```

## Implementation Notes
- **Composition Complexity**: Composite queries currently perform a Cartesian product of sub-matches ($O(\prod |x_i|)$), which is then filtered by connectivity constraints.
- **Type Safety**: The use of `State` as a generic parameter prevents the accidental use of unbound pattern wires in logic expecting design-matched cells.