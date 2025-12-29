# TODO

## DSL & Query Expressivity

- [ ] **Lambda / Procedural Queries**
    - *Goal:* Introduce a "Lambda" query type that allows defining structure via code/logic rather than static netlists.
    - *Use Case:* Recognizing recursive structures like "OR-trees" of arbitrary depth (e.g., for CWE-1234 unlock logic) or DeMorgan equivalent structures without explicitly defining every permutation as a variant.
    - *Notes:* Could the core netlist matching algorithm be ported to this implementation? i.e. could the graph matching be implemented as a Lambda query of sorts?

- [ ] **Negative Composition ("NOT" Logic)**
    - *Goal:* Allow queries to specify the *absence* of a structure or connection (e.g., "Match A that is NOT connected to B").
    - *Challenge:* Prevent combinatorial explosion: avoid matching "everything in the universe that isn't B", or "all pairs of A and B where A is not connected to B".

- [ ] **Standardized Interfaces For All Query Types**
    - *Goal:* Decouple implementation variants from their interfaces.
    - *Implementation:* Define standard accessors (`enable()`, `data_in()`) for all query types. For variants (Async vs Sync) they must implement the same accessors (or return a None variant), allowing parent queries to interact with the interface rather than specific struct fields.

- [ ] **FSM Recognition**
    - *Goal:* Enable detection of FSM-related vulnerabilities.
    - *Prerequisite:* 
        1. The user should be able to specify the FSM's state registers, and the tool should automatically extract and analyze the transition logic cloud.
        2. Need the use of "Lambda" matching to parse combinational logic cones driving registers.

- [ ] **User-Guided Matching**
    - *Goal:* Allow the user to somehow show which cells or wires match for a certain query.
    - *Implementation:* This could be accomplished perhaps with a signal / module path or perhaps a metadata piece in the file.

## User Interface & API Architecture

- [ ] **Simplify Executor / Driver**
    - *Current:* The current driver handles loading of haystacks / keeping them in memory, but does no query optimization / parallelization.
    - *Goal:* Ideally query execution could happen inside the driver, and the macro just calls the driver and returns the results.
    - *Concept:* This is not syntactically valid but conveys the API it should satisfy.
        ```rust
        fn query<Query>(q: Query<Search>) -> Vec<Query<Result>> {
            todo!()
        }
        ```

- [ ] **Simplify Macros UI**
    - *Current:* The current macro system is complex and requires too much understanding of the underlying mechanism.
    - *Goal:* Almost the entire underlying mechanism should be opaque to the user except for the query layer. As well as the lambda layer should that be implemented.

## Research & Core Algorithms

- [ ] **Hierarchical Netlist Analysis**
    - *Current:* The engine requires a flattened netlist (via Yosys `flatten` pass).
    - *Goal:* Implement a stack-based traversal mechanism to perform matching on unflattened module hierarchies.

- [ ] **Optimized Composite Matching**
    - *Current:* Composite queries often rely on the Cartesian product of sub-matches ($O(|x_1| * |x_2| * \dots * |x_k|)$), filtered dynamically.
    - *Goal:* Implement a graph-topology-based constraint solver for joining sub-query results.

- [ ] **GPU Acceleration Feasibility**
    - *Current:* The current backtracking algorithm is parallelized and can execute on multiple CPU cores.
    - *Goal:* Investigate porting the atomic netlist subgraph isomorphism kernel to GPU.

- [ ] **Memory Organization**
    - *Current:* It is unclear currently if the query algorithm is the most efficient it can be in regards to the memory taken up by queries.
    - *Goal:* Investigate if there are any good optimization strategies for graph storage during query execution.

## Internal Implementation & Refactoring

- [ ] **Simplify Macro Codegen**
    - *Current:* The current macro system is complex and extending it takes a lot of effort.
    - *Goal:* 
        - Ideally the flow of codegen should look like the generated file to make it easier to follow.
        - Simplifying the Driver / Executor layer would simplify the macro generation by a lot.

- [ ] **Reduce Boilerplate (Macro Expansion)**
    - *Goal:* Reduce the Lines of Code (LoC) required to define a query.
    - *Action:* Refactor `netlist!` and `composite!` macros to infer more information or use more concise syntax.

- [ ] **Compile Time Optimization**
    - *Issue:* Heavy use of generics and monomorphization leads to long compile times.
    - *Action:* Investigate using `dyn Trait` dispatch for high-level query orchestration where the runtime overhead is negligible compared to the graph algorithm cost.