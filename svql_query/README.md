# SVQL Query

This crate contains the core query definitions, primitives, and pattern matching logic for the SystemVerilog Query Language (SVQL). It defines how structural patterns are represented, composed, and matched against a hardware netlist.

## Architecture Refactor TODO

We are moving towards a Trait-based variant system to decouple the definition of security concepts (like a "Locked Register") from their specific implementations. This will simplify the macros and allow for cleaner polymorphism.

### 1. Define Behavioral Traits
Instead of hardcoding port mappings in the variant macro, we will define traits that enforce the interface.

- [ ] Create `src/traits/behaviors.rs`.
- [ ] Define `LockedRegisterBehavior<S: State>`:
    ```rust
    pub trait LockedRegisterBehavior<S: State>: WithPath<S> {
        fn clk(&self) -> &Wire<S>;
        fn data_in(&self) -> &Wire<S>;
        fn data_out(&self) -> &Wire<S>;
        fn enable_wire(&self) -> &Wire<S>; // Abstracts 'write_en' vs 'sel'
        fn reset(&self) -> Option<&Wire<S>>; // Handles optional resets
    }
    ```

### 2. Update `netlist!` Macro
Expand the macro to allow implementing these traits inline, mapping internal ports to the trait interface.

- [ ] Add `implements` block support to `netlist!`.
- [ ] Support mapping logic (e.g., `enable_wire: sel`).
- [ ] Support optional wrapping (e.g., `reset: Some(resetn)`).

**Example Target Syntax:**
```rust
netlist! {
    name: AsyncDffMuxEnable,
    // ... ports ...
    implements: [
        LockedRegisterBehavior {
            clk: clk,
            enable_wire: sel, // Mapping 'sel' to 'enable_wire'
            reset: Some(resetn)
        }
    ]
}
```

### 3. Update `composite!` Macro
Expand the macro to allow composites to implement behavior traits by exposing wires from their sub-components.

- [ ] Add `implements` block support to `composite!`.
- [ ] Support dot-notation access to sub-components (e.g., `enable_wire: logic_block.y`).

**Example Target Syntax:**
```rust
composite! {
    name: ComplexLockedReg,
    subs: [ dff: DffAny, logic: LogicBlock ],
    // ... connections ...
    implements: [
        LockedRegisterBehavior {
            clk: dff.clk,
            enable_wire: logic.y, // Exposing internal logic wire
            reset: None
        }
    ]
}
```

### 4. Create `variant_behavior!` Macro
Replace the existing `variant!` macro with a simplified version that generates an Enum and delegates trait calls to the inner variants.

- [ ] Create `variant_behavior!` macro.
- [ ] Remove `common_ports` mapping logic (responsibility moved to primitives).
- [ ] Generate `impl Trait for Enum` that matches on `self` and calls the inner value's trait method.

**Example Target Syntax:**
```rust
variant_behavior! {
    enum_name: LockedRegister,
    trait_name: LockedRegisterBehavior,
    interface: [
        fn enable_wire(&self) -> &Wire<S>;
        // ... other methods
    ],
    variants: [
        (AsyncEn, AsyncDffEnable),
        (AsyncMux, AsyncDffMuxEnable)
    ]
}
```

### 5. Refactor Existing Patterns
- [ ] Refactor `LockedRegister` to use the new system.
- [ ] Update `Cwe1234` to use `LockedRegisterBehavior::enable_wire()` instead of accessing struct fields directly.
```