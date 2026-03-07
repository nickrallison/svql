# svql_cli

Command-line interface for the Structural Verilog Query Language (SVQL). Search hardware designs for security vulnerabilities and structural patterns.

## Installation

```bash
# Build from source
cargo build --release -p svql_cli

# Run directly
cargo run -p svql_cli -- [OPTIONS]
```

## Quick Start

```bash
# Run all registered queries on default design (HummingbirdV2)
cargo run -p svql_cli

# Run specific query on a custom design
cargo run -p svql_cli -- -f design.json -m top_module -q cwe1234

# List available queries
cargo run -p svql_cli -- --list-queries
```

## Available Queries

The following security queries are registered by default:

| Query | Description | CWE |
|-------|-------------|-----|
| `cwe1234` | Hardware Internal or Debug Modes Allow Override of Locks | CWE-1234 |
| `cwe1271` | Uninitialized Value on Reset for Registers Holding Security Settings | CWE-1271 |
| `cwe1280` | Access Control Check Implemented After Asset is Accessed | CWE-1280 |

## Usage Examples

### Basic Usage

Run all queries on the default design (HummingbirdV2 e203_soc_top):
```bash
cargo run -p svql_cli
```

Run on a specific design:
```bash
cargo run -p svql_cli -- \
  -f examples/fixtures/larger_designs/json/hackatdac21/openpiton_tile.json \
  -m tile
```

### Query Selection

Run a specific query:
```bash
cargo run -p svql_cli -- -q cwe1234
```

Run multiple specific queries:
```bash
cargo run -p svql_cli -- -q cwe1234 -q cwe1280
```

List all available queries:
```bash
cargo run -p svql_cli -- --list-queries
```

### Performance Profiling

Enable timing and memory profiling:
```bash
cargo run -p svql_cli -- --profile
```

Example output:
```
=== Results for Cwe1234 ===
Matches found: 1
Execution time: 59.37s
Memory delta: 1444.30 MB

╔══════════════════════════════════════════════════════════════════════╗
║                    SVQL Query Performance Summary                    ║
╠══════════════════════════════════════════════════════════════════════╣
║ Query                Matches       Time (ms)         Memory (MB) ║
╠══════════════════════════════════════════════════════════════════════╣
║ Cwe1234                    1           59374              1444.30 ║
╚══════════════════════════════════════════════════════════════════════╝

Aggregate Statistics:
  Total matches: 1
  Total execution time: 59.37s
  Total memory delta: 1444.30 MB
```

### Detailed Results

Print detailed match information (limited to first 10 matches per query):
```bash
cargo run -p svql_cli -- -q cwe1234 --print-results
```

Example output:
```
=== Results for Cwe1234 ===
Matches found: 1

╔══════════════════════════════════════════════════════════════════════╗
║ Results for: Cwe1234                                                 ║
╚══════════════════════════════════════════════════════════════════════╝

--- Cwe1234 (1 matches) ---
Cwe1234 (svql_query_lib::security::cwe1234::Cwe1234)
|-- unlock_logic (UnlockLogic)
|   |-- top_and (AndGate: And)
|   |   |-- a (Input: CellId: 12345): design.v:45
|   |   |-- b (Input: CellId: 12346): design.v:46
|   |   +-- y (Output: CellId: 12347): design.v:47
|   |-- rec_or (RecOr)
|   |   +-- ...
|   +-- not_gate (NotGate: Not)
|       |-- a (Input: CellId: 12348): design.v:50
|       +-- y (Output: CellId: 12349): design.v:51
+-- locked_register (LockedRegister)
    +-- ...
```

### Parallel Execution

Enable multi-threaded execution for faster processing on large designs:
```bash
cargo run -p svql_cli -- -p --profile
```

### Match Length Constraints

Control how strictly pattern length must match:

```bash
# Stop after first match (fastest)
cargo run -p svql_cli -- --match-length first

# Needle must be subset of haystack (default)
cargo run -p svql_cli -- --match-length needle-subset-haystack

# Exact length match required (strictest)
cargo run -p svql_cli -- --match-length exact
```

### Raw Import

Skip Yosys processing for pre-processed JSON netlists:
```bash
cargo run -p svql_cli -- --use-raw-import -f preprocessed.json
```

## Command-Line Options

```
Usage: svql [OPTIONS]

Options:
  -f, --design-path <DESIGN_PATH>
          Path to the design file (Verilog, RTLIL, or JSON)
          [default: examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json]
  
  -m, --design-module <DESIGN_MODULE>
          Name of the top-level module in the design
          [default: e203_soc_top]
  
      --use-raw-import
          Use raw import (skip Yosys processing)
  
  -p, --parallel
          Enable parallel execution
  
      --match-length <MATCH_LENGTH>
          Set match length constraint
          [default: needle-subset-haystack]
          [possible values: first, needle-subset-haystack, exact]
  
  -q, --query <QUERY>
          Query types to run (can be specified multiple times).
          If omitted, all registered queries are executed.
          [possible values: cwe1234, cwe1271, cwe1280]
  
      --list-queries
          List available queries and exit
  
      --profile
          Enable profiling output (timing and memory usage)
  
      --print-results
          Print detailed results for all matches (not just summary)
  
  -h, --help
          Print help (see a summary with '-h')
  
  -V, --version
          Print version
```

## Environment Variables

- `RUST_LOG`: Control logging verbosity (e.g., `RUST_LOG=debug`, `RUST_LOG=info`)

## Performance Tips

1. **Use `--match-length first`** for quick scans when you only need to know if a pattern exists
2. **Use `-p` (parallel)** for large designs (>100k gates)
3. **Profile first**: Run with `--profile` to identify slow queries before using `--print-results`
4. **Limit output**: `--print-results` can be verbose; use it with specific queries (`-q`) rather than all queries

## Exit Codes

- `0`: Success (queries executed, results found or not found)
- `1`: Error (design load failure, query execution error, etc.)