use svql_macros::netlist;

netlist! {
    name: Sdffe,
    module_name: "sdffe",
    file: "examples/patterns/basic/ff/verilog/sdffe.v",
    inputs: [clk, d, reset],
    outputs: [q]
}

// ============================================================================
// Basic DFF - $dff cell from Yosys
// ============================================================================
netlist! {
    name: SimpleDff,
    module_name: "simple_dff",
    file: "examples/patterns/basic/ff/rtlil/simple_dff.il",
    inputs: [clk, d],
    outputs: [q]
}

// ============================================================================
// Synchronous Reset DFF - $sdff cell
// ============================================================================
netlist! {
    name: SyncResetDff,
    module_name: "sync_reset_dff",
    file: "examples/patterns/basic/ff/rtlil/sync_reset_dff.il",
    inputs: [clk, d, srst],
    outputs: [q]
}

// ============================================================================
// Asynchronous Reset DFF - $adff cell
// ============================================================================
netlist! {
    name: AsyncResetDff,
    module_name: "async_reset_dff",
    file: "examples/patterns/basic/ff/rtlil/async_reset_dff.il",
    inputs: [clk, d, arst],
    outputs: [q]
}

// ============================================================================
// Synchronous Reset with Enable - $sdffe cell
// ============================================================================
netlist! {
    name: SyncResetEnableDff,
    module_name: "sync_reset_enable_dff",
    file: "examples/patterns/basic/ff/rtlil/sync_reset_enable_dff.il",
    inputs: [clk, d, srst, en],
    outputs: [q]
}

// ============================================================================
// Asynchronous Reset with Enable - $adffe cell
// ============================================================================
netlist! {
    name: AsyncResetEnableDff,
    module_name: "async_reset_enable_dff",
    file: "examples/patterns/basic/ff/rtlil/async_reset_enable_dff.il",
    inputs: [clk, d, arst, en],
    outputs: [q]
}

// ============================================================================
// DFF with Enable - $dffe cell
// ============================================================================
netlist! {
    name: EnableDff,
    module_name: "enable_dff",
    file: "examples/patterns/basic/ff/rtlil/enable_dff.il",
    inputs: [clk, d, en],
    outputs: [q]
}
