# svql_query

## Purpose

The svql_query crate implements the query DSL (Domain Specific Language) for hardware pattern matching. It provides abstractions for defining and executing queries against hardware designs. Users can compose complex queries using predefined netlist patterns or create custom composite queries that combine multiple patterns with connectivity constraints.

Key features:
- Type-safe query definitions using Rust's type system
- Support for both simple netlist patterns and complex composite queries
- Binding management between pattern and design elements
- Instance path tracking for hierarchical query results
- Enum-based queries for matching multiple pattern variants
- Connection validation between query components

The crate serves as the primary interface for users to define what they want to search for in their hardware designs.