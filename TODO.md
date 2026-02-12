# TODO

## DSL & Query Expressivity

- [ ] **Lambda / Procedural Queries**
    - *Goal:* Introduce a "Lambda" query type that allows defining structure via code/logic rather than static netlists.
    - *Use Case:* Recognizing recursive structures like "OR-trees" of arbitrary depth (e.g., for CWE-1234 unlock logic) or DeMorgan equivalent structures without explicitly defining every permutation as a variant.
    - *Notes:* Could the core netlist matching algorithm be ported to this implementation? i.e. could the graph matching be implemented as a Lambda query of sorts?

- [ ] **Negative Composition ("NOT" Logic)**
    - *Goal:* Allow queries to specify the *absence* of a structure or connection (e.g., "Match A that is NOT connected to B").
    - *Challenge:* Prevent combinatorial explosion: avoid matching "everything in the universe that isn't B", or "all pairs of A and B where A is not connected to B".

- [ ] **FSM Recognition**
    - *Goal:* Enable detection of FSM-related vulnerabilities.
    - *Prerequisite:* 
        1. The user should be able to specify the FSM's state registers, and the tool should automatically extract and analyze the transition logic cloud.
        2. Need the use of "Lambda" matching to parse combinational logic cones driving registers.

- [ ] **User-Guided Matching**
    - *Goal:* Allow the user to somehow show which cells or wires match for a certain query.
    - *Implementation:* This could be accomplished perhaps with a signal / module path or perhaps a metadata piece in the file.

## Research & Core Algorithms

- [ ] **Hierarchical Netlist Analysis**
    - *Current:* The engine requires a flattened netlist (via Yosys `flatten` pass).
    - *Goal:* Implement a stack-based traversal mechanism to perform matching on unflattened module hierarchies.

- [ ] **GPU Acceleration Feasibility**
    - *Current:* The current backtracking algorithm is parallelized and can execute on multiple CPU cores.
    - *Goal:* Investigate porting the atomic netlist subgraph isomorphism kernel to GPU.

## Internal Implementation & Refactoring

- [ ] **Compile Time Optimization**
    - *Issue:* Heavy use of generics and monomorphization leads to long compile times.
    - *Action:* Investigate using `dyn Trait` dispatch for high-level query orchestration where the runtime overhead is negligible compared to the graph algorithm cost.