# svql_driver

## Purpose

The svql_driver crate provides design loading and management capabilities for the svql project. It handles the integration with Yosys to parse Verilog designs and convert them into the internal netlist representation. The driver maintains a registry of loaded designs to avoid redundant parsing and provides a context for query execution. It acts as the interface between the file system and the query engine.

Key responsibilities:
- Loading Verilog designs via Yosys
- Managing design registry to avoid duplicate loading
- Providing design contexts for queries to get around lifetime issues
- Handling file path resolution and module extraction