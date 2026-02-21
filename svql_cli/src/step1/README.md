### README.md
#### Grammar
Primitives match single cells in the design index.
```rust
define_primitive!(
    RustTypeName, 
    PrjunnamedCellKind, 
    [(port_name, direction), ...]
);
```

#### Example
```rust
define_primitive!(AndGate, And, [(a, input), (b, input), (y, output)]);
```

#### Directions
1. Open `svql_cli/src/main.rs`.
2. Replace `Cwe1234` with `AdcGate` in the `run_query` call and the `store.get` call.
3. Run the tool to find all hardware adders/subtractors.
