# svql_macros: DSL Code Generation

**Purpose**  
Compile-time procedural macro system that transforms declarative Rust type definitions into executable pattern-matching logic. Generates trait implementations for the SVQL query engine, eliminating the need for hand-written traversal algorithms.

## Core Derive Macros

| Macro | Target | Generates |
|-------|--------|-----------|
| `#[derive(Netlist)]` | `struct` | Subgraph isomorphism glue code; schema metadata mapping ports to columnar storage; `netlist_rehydrate()` for result reconstruction. |
| `#[derive(Composite)]` | `struct` | Incremental join algorithm; connectivity constraint validation; dependency preloading logic. |
| `#[derive(Variant)]` | `enum` | Union dispatch code; result concatenation with port remapping; variant discriminant handling. |

## Attribute Semantics

### Netlist Patterns
```rust
#[derive(Netlist)]
#[netlist(file = "path.v", module = "and_gate")]
struct AndGate {
    #[port(input)]  a: Wire,
    #[port(input)]  b: Wire,
    #[port(output)] y: Wire,
}
```
- **`#[netlist(file, module)]`** (required): Specifies source file and target module name.
- **`#[port(direction, rename = "...")]`**: Maps struct fields to module ports. Optional `rename` translates Verilog port names to Rust field names.
- Generates: `impl Netlist` with `const PORTS`, `const MODULE_NAME`, `const FILE_PATH`, and schema initialization.

### Composite Patterns
```rust
#[derive(Composite)]
#[connection(from = ["sub1", "y"], to = ["sub2", "a"])]
#[or_to(from = ["src", "p"], to = [["dst1", "p"], ["dst2", "p"]])]
#[filter(|row, ctx| row.wire("clk") != ctx.clock())]
struct MyComposite {
    #[submodule] sub1: AndGate,
    #[submodule] sub2: OrGate,
    #[alias(output, target = ["sub2", "y"])] result: Wire,
}
```
- **`#[submodule]`**: Marks fields as nested pattern instances (type must implement `Pattern`).
- **`#[connection(from, to, kind)]`**: Mandatory physical connection. `kind = "any"` enables set-membership matching (wire exists in target bundle).
- **`#[or_to]` / `#[or_from]`**: Disjunctive constraints (source must connect to at least one destination).
- **`#[filter(closure)]`**: Custom validation receiving `&Row<Self>` and `&ExecutionContext`. Must return `bool`.
- **`#[alias(dir, target = ["path"])]`**: Exposes submodule ports as parent fields with specified direction.
- Generates: `impl Composite` with `const SUBMODULES`, `const CONNECTIONS` (CNF form), and `compose()` logic.

### Variant Patterns
```rust
#[derive(Variant)]
#[variant_ports(input(clk), input(d), output(q))]
enum AnyDff {
    #[map(clk = ["clk"], d = ["data"], q = ["q"])] 
    Basic(DffBasic),
    #[map(clk = ["clk"], d = ["d"], q = ["out"])] 
    Complex(CustomDff),
}
```
- **`#[variant_ports(...)]`** (required): Declares common interface ports shared across all arms.
- **`#[map(port = ["path"])]`** (per arm): Maps variant-specific port names to the common interface.
- Generates: `impl Variant` with `const PORT_MAPPINGS`, `const VARIANT_ARMS`, and `concatenate()` logic.

## Implementation Design

**Technology Stack**  
- `proc-macro2`: Token manipulation and generation.
- `syn`: Rust syntax parsing (uses `DeriveInput`, `Attribute` parsing).
- `quote`: Quasi-quotation for code generation.

**Generation Pipeline**
1. **Parse**: Extract struct/enum fields and attributes using `syn`.
2. **Schema Construction**: Build `PatternSchema` definitions (columnar storage layout) as static `OnceLock` initializers.
3. **Trait Implementation**: Emit `impl PatternInternal<Kind>` for the specific kind (Netlist/Composite/Variant) containing:
   - `const EXEC_INFO`: Type metadata and search function pointer for dependency graphs.
   - `search_table()`: Dispatch to appropriate search logic.
   - `rehydrate()`: Reconstruct Rust structs from `Row` columnar data.
   - `preload_driver()`: Load dependency designs into the driver cache.

**Key Internal Types** (used during generation)
- `PathSelector`: Parses hierarchical paths like `["sub", "port"]` into traversable structures.
- `ConnectionKind`: `Exact` (wire equality) vs `AnyInSet` (membership in wire bundle).
- `Submodule` / `Alias`: Metadata structs for composite field handling.

## Integration with Ecosystem

- **Output Target**: Generates code that implements traits defined in `svql_query`.
- **Dependency**: Generated `impl` blocks reference `svql_query::prelude::*` and `svql_common::*`.
- **Usage**: `svql_query_lib` uses these macros to define CWE patterns; end users import the macros to define custom queries.

## Complete Example

```rust
// 1. Atomic primitive (external Verilog module)
#[derive(Netlist)]
#[netlist(file = "gates.v", module = "and")]
struct AndGate { 
    #[port(input)] a: Wire, 
    #[port(input)] b: Wire, 
    #[port(output)] y: Wire 
}

// 2. Hierarchical composition
#[derive(Composite)]
#[connection(from = ["and1", "y"], to = ["and2", "a"])]
struct AndChain {
    #[submodule] and1: AndGate,
    #[submodule] and2: AndGate,
    #[alias(input, target = ["and1", "a"])] input_a: Wire,
    #[alias(output, target = ["and2", "y"])] output_y: Wire,
}

// 3. Polymorphic choice
#[derive(Variant)]
#[variant_ports(input(d), output(q))]
enum AnyDff {
    #[map(d = ["d"], q = ["q"])] Basic(Dff),
    #[map(d = ["data"], q = ["out"])] Complex(CustomDff),
}
```

The macros generate all boilerplate—schema definitions, dependency graphs, join planners, and result rehydration—allowing complex hardware queries via simple type declarations.