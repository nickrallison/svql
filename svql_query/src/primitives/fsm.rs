use svql_macros::netlist;

// Primitive: One-hot state register (e.g., 4-state DFF array; width-agnostic via subgraph).
netlist! {
    name: OneHotStateReg,
    module_name: "onehot_state_reg",
    file: "examples/patterns/security/fsm/verilog/onehot_state_reg.v",  // See fixtures below
    inputs: [clk, resetn, next_state],
    outputs: [state]  // Multi-bit output for states
}

// Primitive: Transition mux/case (next_state selection based on current state + conditions).
netlist! {
    name: TransitionMux,
    module_name: "transition_mux",
    file: "examples/patterns/security/fsm/verilog/transition_mux.v",  // See fixtures below
    inputs: [state, cond_a, cond_b],  // Conditions for transitions
    outputs: [next_state]
}
