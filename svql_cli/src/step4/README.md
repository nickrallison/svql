# Step 4: Composites

## Grammar
Composites combine multiple sub-patterns. Connectivity is defined using struct-level attributes.

**Connection Attributes:**
- `#[connection(from = ["sub1", "p"], to = ["sub2", "p"])]`: Mandatory physical connection.
- `#[connection(from = [...], to = [...], kind = "any")]`: Set membership. Source wire must exist within a target `WireArray` (e.g., `LogicCone` inputs).
- `#[or_to(from = ["src", "p"], to = [["dst1", "p"], ["dst2", "p"]])]`: Source must connect to at least one of the listed destinations.
- `#[or_from(from = [["src1", "p"], ["src2", "p"]], to = ["dst", "p"])]`: Destination must be driven by at least one of the listed sources.
- `#[filter(closure)]`: Custom validation logic. Receives `&Row<Self>` and `&ExecutionContext`.

**Field Attributes:**
- `#[submodule]`: Identifies a nested pattern field.
- `#[alias(direction, target = ["sub", "port"])]`: Exposes a submodule port as a field on the parent. Directions: `input`, `output`, `inout`.

## Example
```rust

fn other_filter(row: &Row<Self>, ctx: &ExecutionContext) -> bool {
    let clk1: Wire = row.resolve(Selector::static_path(&["ha", "clk"]), ctx);
    let clk2: Wire = row.resolve(Selector::static_path(&["out_reg", "clk"]), ctx);
    clk1 != clk2
}

#[derive(Debug, Clone, Composite)]
// Standard connection
#[connection(from = ["ha", "sum"], to = ["out_reg", "d"])]
// Set membership (is ha.cout one of the inputs to the logic cone?)
#[connection(from = ["ha", "cout"], to = ["cone", "leaf_inputs"], kind = "any")]
// Commutative OR (ha.sum connects to either input A or B of the gate)
#[or_to(from = ["ha", "sum"], to = [["gate", "a"], ["gate", "b"]])]
// Multiple possible sources
#[or_from(from = [["gate", "y"], ["other", "y"]], to = ["out_reg", "en"])]
// Custom filter (e.g., ensure two submodules don't share the same clock)
#[filter(|row, ctx| {
    let clk1 = row.resolve(Selector::static_path(&["ha", "clk"]), ctx);
    let clk2 = row.resolve(Selector::static_path(&["out_reg", "clk"]), ctx);
    clk1 != clk2
})]
// Can also write filter using function name
#[filter(other_filter)]
pub struct ExampleComposite {
    #[submodule] 
    pub ha: HalfAdder,
    #[submodule] 
    pub cone: LogicCone,
    #[submodule] 
    pub gate: AndGate,
    #[submodule] 
    pub out_reg: DffAny,
    #[alias(output, target = ["out_reg", "q"])] 
    pub data_out: Wire,
}
```

## Directions
1. Define `FullAdderComposite`.
2. Instantiate two `HalfAdder` submodules (`ha1`, `ha2`) and one `OrGate` (`final_or`).
3. Use `#[connection]` to link `ha1.sum` to `ha2.a`.
4. Use `#[connection]` to link `ha1.carry` to `final_or.a` and `ha2.carry` to `final_or.b`.
5. Use `#[alias]` to expose the top-level ports: `a`, `b`, `cin`, `sum`, and `cout`.
