# SVQL (SystemVerilog Query Language)

## Overview
SVQL is a framework for structural pattern matching in hardware netlists. 
It provides a domain-specific language (DSL) to define hierarchical queries 
in terms of composition of hardware and executes them against flattened netlists 
using a specialized subgraph isomorphism engine.

## Query Library
SVQL provides a library of structural patterns targeting a small set of Common Weakness Enumerations (CWEs). 
Thus far, queries have been written for CWE-1234, CWE-1271, and CWE-1280. 
These were chosen for the ability to express them as structural patterns with a hierarchy. 

### CWE-1234: Internal or Debug Modes Allow Override of Locks
This query was chosen because it is quite simple to express the structure that underlies
this bug as a set of faulty unlock logic feeding into a locked register.

```rust
#[composite]
pub struct Cwe1234<S: State> {
    #[path]
    pub path: Instance,
    #[submodule]
    pub unlock_logic: UnlockLogic<S>,
    #[submodule]
    pub locked_register: LockedRegister<S>,
}

impl<S: State> Topology<S> for Cwe1234<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        ctx.connect(
            Some(&self.unlock_logic.top_and.y),
            self.locked_register.write_en(),
        );
    }
}
```

### CWE-1271: Uninitialized Value on Reset for Security Registers
This query is more challenging to express as just a structural pattern.
Part of the bug is an uninitialized register which can be detected by any synthesis tool,
but another part is detecting the value it holds is used for security sensitive tasks which is more challenging to do.
The plan is to implement it in a future version by letting humans tag which cells match for a specific query. 
That way humans could define which registers are security specific and those could be added into this query.

```rust
#[variant(ports(clk, data_in, data_out))]
pub enum Cwe1271<S: State> {
    #[variant(map(clk = "clk", data_in = "data_in", data_out = "data_out"))]
    WithEnable(UninitRegEn<S>),
    #[variant(map(clk = "clk", data_in = "data_in", data_out = "data_out"))]
    Basic(UninitReg<S>),
}
```

### CWE-1280: Access Control Check Implemented After Asset Access
This bug was chosen because also it is quite amenable to structural analysis.
This pattern identifies a locked register enabled by an access checking structure 
where an intermediate register causes a signal delay, leading to stale access checks.

```rust
#[composite]
pub struct Cwe1280<S: State> {
    #[path]
    pub path: Instance,
    #[submodule]
    pub delayed_grant_access: DelayedGrantAccess<S>,
    #[submodule]
    pub locked_reg: LockedRegister<S>,
}

impl<S: State> Topology<S> for Cwe1280<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        ctx.connect(
            self.delayed_grant_access.reg_any.q(),
            self.locked_reg.write_en(),
        );
    }
}
```

## Validation Data

The framework is validated against open-source hardware designs and security competition benchmark. This was done to ensure the tool performs well, in addition to finding true positive & false positive instances of the bugs.

- **Hack@DAC18 & Hack@DAC21**: These modules contain a large set of manually inserted bugs, providing a ground truth for query testing.
- **HummingbirdV2**: A real-world RISC-V core used to test performance on complex, non-buggy designs.


## System Architecture
The workspace is organized into functional layers that separate query definition from the underlying graph algorithms.

| Layer | Crate | Responsibility |
| :--- | :--- | :--- |
| **DSL** | `svql_macros` | Procedural macros for structural pattern definition (`netlist`, `composite`, `variant`). |
| **Orchestration** | `svql_query` | Type-level state management (`Search` vs `Match`) and query abstractions. |
| **Management** | `svql_driver` | Design ingestion and resource management. |
| **Kernel** | `svql_subgraph` | Subgraph isomorphism implementation. |
| **Infrastructure** | `svql_common` | External tool bindings (Yosys), shared configurations, and test utilities. |

## Execution Flow
1.  **Definition**: Users define queries using `svql_macros` & `svql_query` to compose primitives into complex structures, which are then executed.
2.  **Ingestion**: `svql_driver` invokes Yosys to ingest netlists
4.  **Execution**: `svql_query` orchestrates the search dispatches subgraph queries to `svql_subgraph`, which identifies mappings via a backtracking algorithm.
5.  **Reporting**: Results are bound to the `Match` state which provides source-level traceability.

## Workspace Structure
- `svql_common/`: Yosys management, and shared configuration.
- `svql_driver/`: Thread-safe design management and lifetime management.
- `svql_macros/`: Procedural macro implementations for the DSL attributes.
- `svql_query/`: Query logic, traits, and primitive library.
- `svql_subgraph/`: Backtracking subgraph isomorphism algorithm.
- `prjunnamed/`: External netlist abstraction layer.

## Requirements
- Requires `yosys` in the system `PATH`
- Certain designs (e.g., Hack@DAC21) will require TabbyCAD for verific support if you want to compile them from scratch. A compressed json netlist has been included for them in the repo so this is not required by default.

## Usage Example
```rust
use svql_query::{Search, State, Instance, traits::*};
use svql_driver::Driver;

// 1. Define a hierarchical pattern
#[composite]
pub struct MyQuery<S: State> {
    #[path] path: Instance,
    #[submodule] gate: AndGate<S>,
    #[submodule] reg: Sdffe<S>,
}

impl<S: State> Topology<S> for MyQuery<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // Define internal connectivity constraints
        ctx.connect(Some(&self.gate.y), Some(&self.reg.d));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let driver = Driver::new_workspace()?;
    let config = Config::default();

    // 2. Load the target design (haystack)
    let (key, design) = driver.get_or_load_design(
        "path/to/design.v",
        "top_module",
        &config.haystack_options
    )?;

    // 3. Prepare context and execute
    let context = MyQuery::<Search>::context(&driver, &config.needle_options)?
        .with_design(key.clone(), design);

    let query = MyQuery::<Search>::instantiate(Instance::root("q"));
    let matches = query.query(&driver, &context, &key, &config);

    println!("Found {} matches", matches.len());
    Ok(())
}
```

## Implementation Notes
- **Complexity**: Subgraph isomorphism is NP-complete; SVQL mitigates this using hardware-specific heuristics such as cell types, and fan-in degree.
- **Parallelism**: The matcher supports parallelizing the search across independent candidate branches via the use of `rayon`.
- **Memory**: Large designs require a significant amount of memory to store the matches of a search when many matches are found.