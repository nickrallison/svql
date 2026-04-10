# svql_query_lib: Pattern Library

**Purpose**  
Curated collection of production-ready hardware patterns and security vulnerability detectors. Provides reusable implementations of common hardware primitives (gates, flip-flops) and Common Weakness Enumeration (CWE) queries built atop the SVQL framework. Serves as the primary user-facing library for security analysis.

## Module Structure

### `primitives`
Standard hardware components defined via `svql_macros`:
- **Gates**: `AndGate`, `OrGate`, `XorGate`, `NotGate`, `MuxGate`, `AdcGate`, etc.
- **Arithmetic**: `MulGate`, `UDivGate`, `SDivTruncGate`, `ULtGate`, etc.
- **Flip-Flops**: `Dff`, `Dffe`, `Sdffe`, `Adffe`, `DffAny` (variants with/without enable/reset).
- **Recursive Structures**: `RecAnd`, `RecOr` (tree patterns for AND/OR gate chains), `LogicCone` (arbitrary combinational logic trees).

### `security`
CWE-specific composite patterns demonstrating hierarchical query composition:
- **`cwe1234`**: Hardware Internal/Debug Modes Allow Override of Locks.
  - Detects registers with enable inputs driven by unlock logic containing debug bypass signals.
  - Uses `UnlockLogic` (composite of AND/OR/NOT gates) and `DffEnable` (variant of DFF implementations).
- **`cwe1271`**: Uninitialized Value on Reset for Security-Sensitive Registers.
  - Variant pattern matching DFFs without reset/clear signals.
- **`cwe1280`**: Access Control Check Implemented After Asset is Accessed.
  - Composite detecting stale access checks where grant logic feeds a delay register before the protected register.

### `experimental`
Development patterns for advanced features:
- **CDC Detection**: Clock Domain Crossing violation patterns using `DffAny` and `LogicCone` with clock inequality filters.
- **Sequential Patterns**: `SdffeThenAnd` demonstrating flip-flop to combinational logic chains.

## Key Pattern Examples

### Primitive Definition
```rust
// From primitives/gates.rs
svql_query::define_primitive!(
    AndGate, And, 
    [(a, input), (b, input), (y, output)]
);
```
Expands to a struct with `Wire` fields and implements `Primitive` trait for direct cell-kind matching.

### CWE Composite
```rust
// From security/cwe1234/mod.rs
#[derive(Debug, Clone, Composite)]
#[connection(from = ["unlock_logic", "unlock"], to = ["dff_enable", "write_en"])]
pub struct Cwe1234 {
    #[submodule] pub unlock_logic: UnlockLogic,
    #[submodule] pub dff_enable: DffEnable,
}
```
Defines a security vulnerability as the connection between two sub-patterns.

### Variant Pattern
```rust
// From security/primitives/dff_enable.rs
#[derive(Debug, Clone, Variant)]
#[variant_ports(input(clk), input(data_in), output(data_out), input(resetn), input(write_en))]
pub enum DffEnable {
    #[map(...)] En(DffEnCell),           // Primitive cell
    #[map(...)] AsyncMux(AsyncDffMuxEnable),  // Netlist implementation
    #[map(...)] SyncMux(SyncDffMuxEnable),    // Alternative netlist
}
```
Unifies multiple DFF-with-enable implementations under a single query interface.

## Integration with Framework

- **Depends on**: `svql_macros` (for pattern definitions), `svql_query` (for execution), `svql_common` (for Wire/Cell types).
- **Used by**: `svql_cli` for command-line scanning tools and end-user security analysis scripts.
- **Extensibility**: Users can define new patterns using the same macro patterns (`define_primitive!`, `define_dff_primitive!`, `#[derive(Composite)]`, etc.) in their own crates.

## Usage Example

```rust
use svql_query::prelude::*;
use svql_query_lib::security::{Cwe1234, Cwe1271};

fn audit_design(driver: &Driver, key: &DriverKey) -> Result<(), Box<dyn Error>> {
    let config = Config::default();
    
    // Run CWE-1234 query
    let store = run_query::<Cwe1234>(driver, key, &config)?;
    if let Some(table) = store.get::<Cwe1234>() {
        println!("CWE-1234 matches: {}", table.len());
        for (_, row) in table.rows() {
            let vuln = Cwe1234::rehydrate(&row, &store, driver, key, &config)?;
            println!("  Unlock logic drives: {:?}", vuln.dff_enable.write_en());
        }
    }
    
    // Run CWE-1271 query  
    let store = run_query::<Cwe1271>(driver, key, &config)?;
    println!("Uninitialized DFFs: {}", store.get::<Cwe1271>().map(|t| t.len()).unwrap_or(0));
    
    Ok(())
}
```

**Design Philosophy**  
Patterns in this crate demonstrate best practices for SVQL query construction: atomic primitives for indivisible hardware cells, variants for implementation-agnostic matching, and composites for vulnerability-specific structural relationships.