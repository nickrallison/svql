### README.md
#### Grammar
Composites join multiple submodules via connectivity constraints.
```rust
#[derive(Composite)]
#[connection(from = ["sub1", "out"], to = ["sub2", "in"])]
pub struct Parent {
    #[submodule] pub sub1: Child1,
    #[submodule] pub sub2: Child2,
    #[alias(input, target = ["sub1", "in"])] pub top_in: Wire,
}
```

#### Directions
1. Create a `FullAdderComposite`.
2. Instantiate two `HalfAdder` submodules (`ha1`, `ha2`) and one `OrGate` (`final_or`).
3. Connect `ha1.sum` to `ha2.a`.
4. Connect `ha1.carry` and `ha2.carry` to the `final_or` inputs.