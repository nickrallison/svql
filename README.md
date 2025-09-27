# svql

## Purpose

svql (SystemVerilog Query Language) is a hardware analysis tool designed for linting and code searching in Verilog designs. It provides pattern matching capabilities to identify specific circuit structures within larger designs using subgraph isomorphism algorithms. The tool enables users to build composable queries that can search for hardware patterns, making it useful for design verification, security analysis, and code comprehension tasks.

### Submodules

- svql_subgraph
	- implements a subgraph isomorphism algorithm 
- svql_driver
	- Is supposed to hold and manage a set of designs to make more performant / deal with lifetimes
	- Holds all designs in Arcs for Easy Cloning & dealing with lifetimes
- svql_query
	- Is supposed to orchestrate, structure the queries the user runs, then marshal the results into the same shape 


## Depends

- `cargo`
- `yosys`
