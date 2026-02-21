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
2. Replace the generics with `AdcWithCarry` in the `run_query` call and the `store.get` call in the `svql_cli/src/main.rs` file.

```rust
// ...
let store = svql_query::run_query::<AdcWithCarry>(&driver, &design_key, &config)?;
// ...
let rows = store
    .get::<AdcWithCarry>()
    .expect("Store should have table")
    .rows()
    .collect::<Vec<_>>();
// ...
```

3. Run the tool to find all hardware adders/subtractors.

`cargo run --bin svql_cli -- --design-path svql_cli/src/step1/primitive_ha_test.v --design-module primitive_ha_test --parallel`
