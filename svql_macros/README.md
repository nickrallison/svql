# svql_macros

## Purpose
Procedural macro library providing the domain-specific language (DSL) for defining structural hardware patterns and automating the implementation of query traits.

## Key Responsibilities
- **Code Generation**: Implements the boilerplate for `Component`, `Query`, `Searchable`, and `Reportable` traits.
- **DSL Attribute Parsing**: Processes specialized attributes such as `#[path]`, `#[submodule]`, and `#[variant(map(...))]` to establish hierarchical relationships and port mappings.

## Core Abstractions
| Type / Trait | Description |
| :--- | :--- |
| `#[netlist]` | Maps a Rust struct to a netlist file, defining atomic pattern primitives. |
| `#[composite]` | Defines hierarchical patterns by composing multiple sub-queries and enforcing connectivity constraints via the `Topology` trait. |
| `#[variant]` | Implements polymorphic matching, allowing a single query point to match against multiple distinct pattern types. |

## Data Flow
- **Input**: Rust item definitions (structs or enums) decorated with SVQL attributes and configuration parameters (e.g., file paths, module names).
- **Output**: Expanded Rust code that integrates with the `svql_query` execution engine and the `svql_subgraph` isomorphism kernel.

## Usage Example
```rust
use svql_query::{State, Wire, Instance, Search};
use svql_macros::{netlist, composite};

#[netlist(file = "gate.v", name = "and_gate")]
pub struct AndGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

#[composite]
pub struct MyPattern<S: State> {
    #[path] path: Instance,
    #[submodule] gate_a: AndGate<S>,
    #[submodule] gate_b: AndGate<S>,
}

impl<S: State> Topology<S> for MyPattern<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        ctx.connect(Some(&self.gate_a.y), Some(&self.gate_b.a));
    }
}
```

## Implementation Notes
- **Namespace Requirements**: Generated code assumes `svql_query` is available in the crate root.
