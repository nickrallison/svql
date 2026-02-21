### README.md
#### Grammar
Variants allow matching one of several different implementations of the same interface.
```rust
#[derive(Variant)]
#[variant_ports(input(clk), output(q))]
pub enum AnyRegister {
    #[map(clk = ["clk"], q = ["q"])]
    TypeA(Dff),
    #[map(clk = ["c"], q = ["out"])]
    TypeB(CustomReg),
}
```

#### Directions
1. Define `AnyFullAdder` as a Variant.
2. Map `FullAdderComposite` (from Step 3) and `AdcGate` (from Step 1) to a common interface.
3. The interface should have inputs `a`, `b`, `cin` and outputs `sum`, `cout`.