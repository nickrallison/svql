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

## Directions
1. Define the `HalfAdder` struct.
2. Use `#[derive(Netlist)]` to enable subgraph matching.
3. Point the `#[netlist]` attribute to `svql_cli/src/step2/half_adder.v` and the module `half_adder`.
4. Map the fields `a`, `b`, `sum`, and `carry` to their respective ports using the `#[port]` attribute.
