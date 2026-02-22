# Step 3: Variants

## Grammar
Variants define a common interface for multiple different implementations. The query engine will union the results of all arms.

**Variant Attributes:**
- `#[variant_ports(input(p1), output(p2))]`: Defines the common interface ports for the variant.
- `#[map(common_p = ["inner_p"])]`: Maps a common interface port to a specific field or port in the underlying pattern.

#### Example
```rust
#[derive(Clone, Debug, Variant)]
#[variant_ports(input(a), input(b), output(y))]
pub enum AnyLogicGate {
    // Direct mapping to a primitive
    #[map(a = ["a"], b = ["b"], y = ["y"])]
    And(AndGate),
    
    // Mapping to a composite with different internal names
    #[map(a = ["in_0"], b = ["in_1"], y = ["out_val"])]
    Complex(MyCustomComposite),
    
    // Mapping where one input is unused in this specific implementation
    #[map(a = ["a"], y = ["y"])]
    Inverter(NotGate),
}
```


## Directions:

**Netlist:**
1. Define `FullAdderFlat` pointing to `full_adder_flat.v`.
2. This demonstrates matching the same logical function implemented with different structural boundaries.

**Variant:**
3. Define `AnyFullAdder` as an enum.
4. Add `Hierarchical` and `Flat` arms.
5. Map all ports to the common interface.