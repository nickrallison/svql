# SVQL Query Construction Guide

Definitions are compiled into an `ExecutionPlan` where sub-components are resolved into a columnar `Store`.

## 1. Netlist Patterns (Atomic)
The `#[derive(Netlist)]` macro maps a struct to an external Verilog, RTLIL, or JSON file.

### Usage
```rust
#[derive(Netlist)]
#[netlist(file = "gate.v", module = "my_and")]
pub struct AndGate {
    #[port(input)]
    pub a: Wire,
    #[port(output, rename = "y_out")]
    pub y: Wire,
}
```

## 2. Composite Patterns (Hierarchical)
The `#[derive(Composite)]` macro combines multiple sub-queries. Connectivity is defined via struct-level and field-level attributes.

### Connection Attributes
- `#[connection(from = [...], to = [...])]`: Mandatory physical connection.
- `#[or_to(from = [...], to = [[...], [...]])]`: Source connects to any listed destination.
- `#[or_from(from = [[...], [...]], to = [...])]`: Any listed source connects to the destination.
- `#[filter(path_or_closure)]`: Custom logic validation for a row.

### Field Attributes
- `#[submodule]`: Identifies a nested pattern field.
- `#[alias(direction, target = [...])]`: Exposes a submodule port as a field on the parent struct.

### Alias Directions
- `input`: Marks the alias as an input port.
- `output`: Marks the alias as an output port.
- `inout`: Marks the alias as bidirectional.

### Usage
```rust
#[derive(Composite)]
#[connection(from = ["gate_a", "y"], to = ["gate_b", "a"])]
pub struct MyComposite {
    #[submodule]
    pub gate_a: AndGate,
    #[submodule]
    pub gate_b: AndGate,
    #[alias(input, target = ["gate_a", "a"])]
    pub external_in: Wire,
}
```

## 3. Variant Patterns (Polymorphic)
The `#[derive(Variant)]` macro creates a type-safe union of different implementations sharing a common interface.

### Usage
```rust
#[derive(Variant)]
#[variant_ports(input(clk), output(q))]
pub enum DffAny {
    #[map(clk = ["clk_in"], q = ["q_out"])]
    Standard(Dff),
    #[map(clk = ["clock"], q = ["data_q"])]
    WithEnable(Dffe),
}
```

## 4. Recursive Patterns (Trees)
Used for structures of indeterminate depth (e.g., OR-trees). These require a manual `Recursive` trait implementation. 
- Matches are computed via fixpoint iteration.
- Results represent the maximal tree rooted at each target cell.

## 5. Built-in Primitives
Located in `svql_query_lib::primitives`:
- `AndGate`, `OrGate`, `NotGate`, `MuxGate`, `XorGate`
- `Dff`, `Sdffe`, `Adffe`, `Sdff`, `Adff`, `Dffe`

## Results Interpretation
Accessing a `Row<T>` provides:
- **Wire**: A rehydrated cell reference with direction metadata.
- **Ref<T>**: A type-safe row index into a submodule table.
- **Metadata**: Pattern-specific data such as tree depth.