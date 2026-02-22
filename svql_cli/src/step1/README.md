# Step 2: Netlists

## Grammar
Netlists map a Rust struct to a specific Verilog or RTLIL module. The query engine uses subgraph isomorphism to find all instances of that module's logic within the design.

**Netlist Attributes:**
- `#[netlist(file = "path/to/file.v", module = "module_name")]`: Specifies the source file and the target module name to match.
- `#[port(direction)]`: Maps a struct field to a module port. Directions: `input`, `output`, `inout`.
- `#[port(direction, rename = "y_out")]`: Maps a struct field to a module port. Renames from the netlist port name "y_out" to the field name.

## Example
```rust
#[derive(Clone, Debug, Netlist)]
#[netlist(file = "svql_cli/src/step2/example.v", module = "my_logic")]
pub struct MyLogicPattern {
    #[port(input)]
    pub clk: Wire,
    #[port(input)]
    pub reset_n: Wire,
    #[port(output, rename = "other_data_out")]
    pub data_out: Wire,
}
```

## Directions:
1. Define `FullAdderHierarchical` using two `HalfAdder` submodules and an `OrGate`.
2. Link `ha1.sum` to `ha2.a`.
3. Link both carries to the `final_or` gate.