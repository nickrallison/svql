### README.md
#### Grammar
Recursive patterns and Logic Cones allow matching arbitrary combinational paths between endpoints.
```rust
#[connection(from = ["src", "q"], to = ["cone", "leaf_inputs"], kind = "any")]
```
The `kind = "any"` constraint checks if a wire exists within a `WireArray` (like the inputs to a logic cone).

#### Directions
1. Create a `CdcViolation` pattern.
2. Use `#[filter]` to ensure the `source.clk` and `dest.clk` are different physical nets.
3. Use `LogicCone` to find paths that pass through combinational gates.