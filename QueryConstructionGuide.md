# SVQL Query Construction Guide

SVQL uses a Rust-based DSL to define hardware patterns. These patterns are matched against flattened netlists using a subgraph isomorphism algorithm.

## Query States

All query components are generic over a `State`.

- `Search`: Used during pattern definition. Wires are unbound.
- `Match`: Used for results. Wires contain references to physical design cells.

## 1. Netlist Queries (Atomic)

The `#[netlist]` macro defines an atomic pattern by mapping a Rust struct to an external hardware file, either Verilog, RTLIL, or JSON. 
RTLIL or JSON is often a better choice as they have more precise semantics and will not change when being read via Yosys, whereas a Verilog file **might change**.

### Usage
Use this for leaf-level primitives or small, fixed-logic blocks.

```rust
#[netlist(file = "path/to/gate.v", name = "my_and_gate")]
pub struct AndGate<S: State> {
    pub a: Wire<S>,
    pub b: Wire<S>,
    #[rename("y_out")]
    pub y: Wire<S>,
}
```

### Key Features
- `file`: Path to the source netlist.
- `name`: The module name inside that file.
- `#[rename("...")]`: Maps a struct field to a specific port name in the netlist.

## 2. Composite Queries (Hierarchical)

The `#[composite]` macro combines multiple sub-queries into a larger structure. It requires implementing the `Topology` trait to define internal connectivity.

### Usage
Use this to build complex structures from simpler components.

```rust
#[composite]
pub struct MyComposite<S: State> {
    #[submodule]
    pub gate_a: AndGate<S>,
    #[submodule]
    pub gate_b: AndGate<S>,
}

impl<S: State> Topology<S> for MyComposite<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // Connect output of gate_a to input of gate_b
        ctx.connect(Some(&self.gate_a.y), Some(&self.gate_b.a));
    }
}
```

### Connection Builder
The `ConnectionBuilder` enforces structural constraints between sub-matches.

- `connect(from, to)`: A mandatory connection. Both wires must be physically connected in the design.
- `connect_any(options)`: A logical OR constraint. At least one pair in the list must be connected. This is useful for commutative inputs or optional paths.

```rust
ctx.connect_any(&[
    (Some(&self.driver.out), Some(&self.receiver.in_a)),
    (Some(&self.driver.out), Some(&self.receiver.in_b)),
]);
```

## 3. Variant Queries (Polymorphic)

The `#[variant]` macro allows a single query point to match different underlying implementations (e.g., different types of flip-flops).

### Usage
Use this when the exact implementation of a component might vary across designs.

```rust
#[variant(ports(clk, d, q))]
pub enum DffAny<S: State> {
    #[variant(map(clk = "clk", d = "d", q = "q"))]
    Sync(Sdffe<S>),
    #[variant(map(clk = "clk", d = "data_in", q = "data_out"))]
    Async(Adffe<S>),
}
```

### Key Features
- `ports(...)`: Defines the common interface for the enum.
- `map(...)`: Maps the common port names to the specific field names of the inner struct.

### 4. Built-in Primitives (Single Gates)

SVQL provides a library of pre-defined primitives in `svql_query::primitives`. These do not require external netlist files because they match directly against cell types exposed by `prjunnamed`.

#### Logic Gates
Standard logic gates use ports `a`, `b`, and `y` (output).

- `AndGate<S>`: 2-input AND.
- `OrGate<S>`: 2-input OR.
- `NotGate<S>`: Inverter (ports `a`, `y`).
- `XorGate<S>`: 2-input XOR.
- `MuxGate<S>`: 2-to-1 Multiplexer (ports `a`, `b`, `sel`, `y`).

#### Arithmetic Gates
Used for data-path matching.

- `AddGate<S>`: Adder.
- `MulGate<S>`: Multiplier.
- `EqGate<S>`: Equality comparator.
- `LtGate<S>`: Less-than comparator.

#### Flip-Flops
Flip-flops are categorized by their control signals, these can also be 

- `Dff<S>`: Basic D-type flip-flop.
- `Dffe<S>`: DFF with Clock Enable (`en`).
- `Sdff<S>`: DFF with Synchronous Reset (`reset`).
- `Sdffe<S>`: DFF with Sync Reset and Clock Enable.
- `Adff<S>`: DFF with Asynchronous Reset (`reset_n`).
- `Adffe<S>`: DFF with Async Reset and Clock Enable.


#### Custom Flip-Flop Primitives
Users can define custom flip-flop queries using the `impl_dff_primitive!` macro. This macro generates a `Pattern` implementation that filters the design's flip-flop cells based on a provided closure.

```rust
impl_dff_primitive!(
    MyCustomDff,
    [clk, d, q, custom_port],
    |ff| ff.has_reset() && !ff.has_enable(),
    "Matches flip-flops with reset but no enable."
);
```

The filter closure receives a `&prjunnamed_netlist::FlipFlop` and should return a `bool`.

### 5. Manual Queries (Procedural)

Sometimes a static netlist or simple composition is insufficient. You can implement the `Pattern` trait manually for complex logic.

#### When to write by hand
- **Recursive Structures**: Matching trees of arbitrary depth (e.g., an OR tree).
- **Algorithmic Filtering**: When matches depend on complex properties not easily expressed as connections.

#### Implementation Example
Manual queries must implement `Hardware`, `Pattern`, and `Matched`.

```rust
impl Pattern for MyManualQuery<Search> {
    type Match = MyManualQuery<Match>;

    fn instantiate(base_path: Instance) -> Self {
        // Initialize search state
    }

    fn context(driver: &Driver, config: &ModuleConfig) -> Result<Context, Box<dyn Error>> {
        // Load required designs into the driver
    }

    fn execute(&self, driver: &Driver, context: &Context, key: &DriverKey, config: &Config) -> Vec<Self::Match> {
        // 1. Run sub-queries
        // 2. Perform custom filtering or recursion
        // 3. Return bound Match objects
    }
}
```

See the `UnlockLogic<S>` query for an  example of this.

### Summary Table

| Macro | Best For | Connectivity |
| :--- | :--- | :--- |
| `#[netlist]` | Atomic gates, fixed RTL modules | Defined by external file |
| `#[composite]` | Hierarchical patterns | `Topology` trait |
| `#[variant]` | Abstractions | Port mapping |
| **Manual** | Recursion, custom logic | Procedural code |