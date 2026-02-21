### README.md
#### Grammar
Netlists match a specific Verilog/RTLIL module using subgraph isomorphism.
```rust
#[derive(Netlist)]
#[netlist(file = "path/to/file.v", module = "module_name")]
pub struct MyPattern {
    #[port(input)]
    pub my_input: Wire,
    #[port(output)]
    pub my_output: Wire,
}
```

#### Example
```rust
#[derive(Netlist)]
#[netlist(file = "logic.v", module = "and_gate")]
pub struct AndGate {
    #[port(input)] pub a: Wire,
    #[port(output)] pub y: Wire,
}
```

#### Directions
1. Define a `HalfAdder` struct.
2. Point it to `examples/fixtures/logic/half_adder.v`.
3. Match the ports: `a`, `b` (inputs) and `sum`, `carry` (outputs).