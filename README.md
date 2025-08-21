# svql

## Purpose

svql (SystemVerilog Query Language) is a hardware analysis tool designed for linting and code searching in Verilog designs. It provides pattern matching capabilities to identify specific circuit structures within larger designs using subgraph isomorphism algorithms. The tool enables users to build composable queries that can search for hardware patterns, making it useful for design verification, security analysis, and code comprehension tasks.

## Brain Thoughts

Above is a project being created to work as a verilog linter / code search tool. It is presently cut up into the following crates

1. prjunnamed_netlist - prjunnamed_json
	1. The actual concrete hardware representation / creation
2. svql_subgraph
	1. An implementation of a subgraph isomorphism algorithm for hardware
		1. Works as a linear search of the least common gate in present in both the needle & haystack
		2. tries to count on the fact that most hardware graphs are very sparse, as well as most ports cannot be swapped e.g. cannot swap en and rst ports on a dff, and also that the graphs are directed.
			1. So far the results have been promising with performance gains when compared against the yosys extract pass 
3. svql_driver
	1. An owned representation of the drivers so if multiple instances of a query (see svql_query) are instantiated, the design only has to be allocated once, and the user doesn't have to worry about smart pointers
4. svql_query
	1. The DSL backing how users can write and compose their own queries
	2. The goal here is to allow a user to build some amount of computation into a query 
	3. Also to back the query system by the type system to make use of Rust's ADTs
		1. For instance, it would be nice to make a query as an enum of different netlists, for example a locked register might be sync or async, and also might use either an enable or a mux, or some other combination. But it would still be helpful to search for all of these and compose them into a vec of enums with each variant containing a different kind of locked register
			1. Also want to be able to make a signal name match a regex which seems to conflict with the goal of backing queries by types, deal with this problem later
	4. Another hope is to orchestrate the queries in a way that deduplicates different queries when run, and passing the results back to both individually
		1. May have to design in a way to forward all queries to a manager which dedups and clones when necessary
		2. This could also be a good opportunity to use Rayon in organized in this way


## Depends

- `cargo`
- `yosys`
