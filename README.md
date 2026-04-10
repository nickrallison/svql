# SVQL: SystemVerilog Query Language

**Purpose**  
A framework for structural pattern matching in hardware netlists. SVQL enables security analysts and hardware designers to algorithmically identify potential security vulnerabilities, design patterns, and structural anomalies in RTL designs using a Rust-based domain-specific language (DSL) and graph-based matching algorithms.

## Overview

SVQL addresses the challenge of identifying hardware security vulnerabilities early in the design cycle when they are least expensive to fix. Unlike traditional static analysis tools that operate on abstract syntax trees (ASTs) and are limited by syntactic representations, SVQL operates on flattened gate-level netlists, enabling it to recognize structurally equivalent but syntactically distinct implementations of vulnerable patterns.

**Key Innovations**
- **DSL-Based Query Authoring**: Define patterns as Algebraic Data Types (ADTs) rather than hand-written traversal algorithms.
- **Graph-Based Matching**: Subgraph isomorphism engine that recognizes structural equivalence across different coding styles.
- **Hierarchical Composition**: Build complex queries by composing atomic patterns (Netlists) into hierarchical structures (Composites) and polymorphic unions (Variants).
- **Production Performance**: Parallel execution with intelligent join planning and caching, capable of analyzing million-gate designs.

## Architecture


**Data Flow**
1. **Ingestion**: `svql_driver` loads Verilog/RTLIL via Yosys, builds `GraphIndex` (topological order + connectivity maps).
2. **Definition**: User defines patterns using `svql_macros` DSL (`#[derive(Netlist)]`, `#[derive(Composite)]`).
3. **Compilation**: Macros generate trait implementations for `svql_query` traits (schema, search logic, rehydration).
4. **Execution**: `svql_query` builds execution DAG, schedules parallel search via `svql_subgraph` (for Netlists) or direct indexing (for Primitives).
5. **Storage**: Results stored in columnar `Table<T>` structures within `Store`, enabling cross-pattern references.
6. **Reporting**: `svql_cli` renders hierarchical match trees, CSV exports, or LaTeX tables.

## Core Concepts

### Pattern Types (ADTs)

| Pattern Kind | Description | Use Case |
|--------------|-------------|----------|
| **Netlist** | Atomic pattern matching an external Verilog/RTLIL module via subgraph isomorphism. | Standard cells, specific module instances. |
| **Primitive** | Direct cell-type match using design index (bypasses subgraph search). | AND gates, DFFs, basic logic elements. |
| **Composite** | Hierarchical composition of sub-patterns with connectivity constraints. | CWEs spanning multiple modules, complex structures. |
| **Variant** | Polymorphic union matching any of several alternative implementations. | DFF-with-enable (primitive OR mux-based). |
| **Recursive** | Self-referential tree patterns built via fixpoint iteration. | Logic cones, gate trees, arbitrary-depth chains. |

### The DSL

Patterns are declared as Rust types with attributes:

```rust
// 1. Atomic primitive
#[derive(Netlist)]
#[netlist(file = "gates.v", module = "and_gate")]
struct AndGate {
    #[port(input)] a: Wire,
    #[port(input)] b: Wire,
    #[port(output)] y: Wire,
}

// 2. Hierarchical composition
#[derive(Composite)]
#[connection(from = ["unlock", "y"], to = ["reg", "write_en"])]
struct Cwe1234 {
    #[submodule] unlock: UnlockLogic,
    #[submodule] reg: DffEnable,
}

// 3. Polymorphic choice
#[derive(Variant)]
#[variant_ports(input(clk), output(q))]
enum AnyDff {
    #[map(clk = ["clk"], q = ["q"])] Basic(Dff),
    #[map(clk = ["clk"], q = ["q"])] WithEnable(DffEn),
}
```

### Execution Model

- **Columnar Storage**: Results stored in `Table<T>` with columns for wires (`Wire`), submodules (`Ref<T>`), and metadata.
- **Lazy Evaluation**: Dependency DAG ensures each sub-query executes exactly once; parallel dispatch via `rayon`.
- **Connectivity Indexing**: Composite queries use pre-computed bipartite indices to avoid Cartesian products.

## Crate Reference

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| **`svql_common`** | Foundational types and Yosys integration | `Wire`, `CellKind`, `GraphIndex`, `YosysModule`, `Config` |
| **`svql_driver`** | Design lifecycle management and caching | `Driver`, `DriverKey`, `DesignContainer` |
| **`svql_macros`** | DSL code generation | `#[derive(Netlist)]`, `#[derive(Composite)]`, `#[derive(Variant)]` |
| **`svql_query`** | Query execution engine and storage | `Table<T>`, `Store`, `ExecutionPlan`, `Pattern` trait |
| **`svql_subgraph`** | Subgraph isomorphism algorithm | `SubgraphMatcher`, `SingleAssignment`, `AssignmentSet` |
| **`svql_query_lib`** | Production pattern library | `Cwe1234`, `Cwe1271`, `AndGate`, `DffEnable`, `LogicCone` |
| **`svql_cli`** | Command-line interface | `Args`, `QueryArg`, `QueryMetrics` |

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/nickrallison/svql
cd svql

# Build CLI (requires Yosys in PATH)
cargo build --release -p svql_cli

# Run tests
cargo test --workspace
```

### Basic Usage

```bash
# Run all registered queries on default design
./target/release/svql_cli

# Run specific CWE query on custom design
./target/release/svql_cli \
  -d "design.json --module top_module" \
  -q cwe1234 \
  --profile

# Export results to CSV and LaTeX
./target/release/svql_cli \
  -d "design.json -m top" \
  -q cwe1234 -q cwe1280 \
  --output-csv results.csv \
  --output-latex results.tex
```

### Programmatic Usage

```rust
use svql_query::prelude::*;
use svql_query_lib::security::cwe1234::Cwe1234;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize driver with workspace root
    let driver = Driver::new_workspace()?;
    
    // Define target design
    let key = DriverKey::new("design.json", "top_module");
    let config = Config::builder()
        .parallel(true)
        .match_length(MatchLength::NeedleSubsetHaystack)
        .build();
    
    // Execute query
    let store = svql_query::run_query::<Cwe1234>(&driver, &key, &config)?;
    
    // Process results
    if let Some(table) = store.get::<Cwe1234>() {
        println!("Found {} matches", table.len());
        
        for (ref, row) in table.rows() {
            let instance = Cwe1234::rehydrate(&row, &store, &driver, &key, &config)?;
            println!("Match: unlock logic drives {:?}", instance.dff_enable.write_en());
        }
    }
    
    Ok(())
}
```

## Performance Characteristics

- **Scalability**: Tested on designs up to 350k+ gates (HummingbirdV2, Hack@DAC).
- **Parallelism**: Multi-threaded execution at query, design, and algorithm levels.
- **Caching**: Design loading and indexing cached per `DriverKey`; query results cached in `TableSlot`.
- **Optimization**: Single-cell primitives bypass subgraph isomorphism; composite queries use connectivity indices to avoid $O(n^k)$ joins.

Example performance (AMD Ryzen 9 7950X, 16 cores):
| Design | Gates | Query | Time (MT) | Memory |
|--------|-------|-------|-----------|--------|
| e203_soc_top | 354k | Cwe1234 | 54ms | 37MB |
| tile (Hack@DAC21) | 124k | Cwe1280 | 209ms | 65MB |

## Development

### Building from Source

Requirements:
- Rust 1.91+ (2024 edition)
- Yosys (for Verilog/RTLIL synthesis)
- Optional: TikV-Jemalloc (enabled in release profile)

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run specific crate tests
cargo test -p svql_subgraph
cargo test -p svql_query
```

### Adding Custom Queries

1. Define pattern in `svql_query_lib` or new crate:
```rust
#[derive(Composite)]
#[connection(from = ["src", "y"], to = ["dst", "a"])]
struct MyPattern {
    #[submodule] src: AndGate,
    #[submodule] dst: OrGate,
}
```

2. Register in `svql_cli/src/args.rs`:
```rust
register_queries!(QueryArg {
    // ... existing queries
    MyPattern => crate::my_pattern::MyPattern,
});
```

3. Rebuild CLI and run:
```bash
cargo run -p svql_cli -- -q my_pattern
```

## Workflow Integration

SVQL fits into a hardware security workflow as an early-stage screening tool:

1. **RTL Development**: Designer writes SystemVerilog.
2. **Synthesis**: Yosys flattens and optimizes design.
3. **Query Execution**: SVQL identifies structural patterns matching CWE signatures.
4. **Manual Review**: Analysts examine reported matches for true positives.
5. **Formal Verification**: Confirmed vulnerabilities undergo formal property checking.

This approach narrows the search space for manual review without requiring formal assertions early in the design cycle.