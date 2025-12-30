# svql_common

## Purpose
Provides shared configuration types, Yosys integration logic, and common test fixtures used across the SVQL workspace crates.

## Key Responsibilities
- **Yosys Management**: Manages the invocation of calls to Yosys to ingest netlist designs in a standardized format.
- **Configuration**: Defines parameters for both design ingestion (e.g., flattening, optimization) and the subgraph isomorphism engine (e.g., match length, deduplication).
- **Test Infrastructure**: Centralizes a library of "needle" and "haystack" definitions to ensure reproducible benchmarking and testing across the workspace.

## Core Abstractions
| Type / Trait | Description |
| :--- | :--- |
| `YosysModule` | Represents a specific module within a source file and handles the transformation pipeline via Yosys. |
| `ModuleConfig` | Encapsulates Yosys-specific passes such as `proc`, `flatten`, `memory`, and `opt_clean`. |
| `Config` | Defines search constraints including `MatchLength` and `Dedupe` strategies. |
| `DesignPath` | Categorizes input files by extension to determine the appropriate Yosys read command. |


## Data Flow
- **Input**: Hardware description files (.v, .il, .json) and user-defined configuration structs.
- **Output**: Processed `prjunnamed_netlist::Design` structures and serialized RTLIL/JSON artifacts.

## Implementation Notes
- **Performance**: The resulting netlists output by this crate should be cached by higher-level abstractions.
- **Constraints**: Requires the `yosys` binary to be present in the system `PATH`, or needs to be passed in manually.