use crate::{State, Wire, instance::Instance};
use svql_macros::{enum_composite, netlist};

// ============================================================================
// Pattern Definitions using netlist! macro
// ============================================================================

netlist! {
    name: AsyncDffEnable,
    module_name: "async_en",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
    inputs: [clk, data_in, resetn, write_en],  // FIXED: Use RTLIL port names
    outputs: [data_out]                         // FIXED: Use RTLIL port name
}

netlist! {
    name: SyncDffEnable,
    module_name: "sync_en",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il",
    inputs: [clk, data_in, resetn, write_en],  // FIXED: Use RTLIL port names
    outputs: [data_out]                         // FIXED: Use RTLIL port name
}

netlist! {
    name: SyncDffMuxEnable,
    module_name: "sync_mux",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il",
    inputs: [clk, data_in, resetn, write_en],  // FIXED: Use RTLIL port names
    outputs: [data_out]                         // FIXED: Use RTLIL port name
}

netlist! {
    name: AsyncDffMuxEnable,
    module_name: "async_mux",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
    inputs: [clk, data_in, resetn, write_en],  // FIXED: Use RTLIL port names
    outputs: [data_out]                         // FIXED: Use RTLIL port name
}

enum_composite! {
    name: RegisterAny,
    variants: [
        (AsyncEn, "async_en", AsyncDffEnable),
        (SyncEn, "sync_en", SyncDffEnable),
        (AsyncMux, "async_mux", AsyncDffMuxEnable),
        (SyncMux, "sync_mux", SyncDffMuxEnable)
    ]
}

// Helper methods for RegisterAny<Match>
impl<S> RegisterAny<S>
where
    S: State,
{
    /// Get the enable wire for connection validation
    /// This is what should connect to the unlock logic output
    pub fn enable_wire(&self) -> &Wire<S> {
        match self {
            RegisterAny::AsyncEn(dff) => &dff.write_en, // FIXED: enable → write_en
            RegisterAny::SyncEn(dff) => &dff.write_en,  // FIXED: enable → write_en
            RegisterAny::AsyncMux(dff) => &dff.write_en, // FIXED: enable → write_en
            RegisterAny::SyncMux(dff) => &dff.write_en, // FIXED: enable → write_en
        }
    }

    /// Get a description of the register type for reporting
    pub fn register_type(&self) -> String {
        match self {
            RegisterAny::AsyncEn(_) => "AsyncDffEnable".to_string(),
            RegisterAny::SyncEn(_) => "SyncDffEnable".to_string(),
            RegisterAny::AsyncMux(_) => "AsyncDffMuxEnable".to_string(),
            RegisterAny::SyncMux(_) => "SyncDffMuxEnable".to_string(),
        }
    }

    pub fn new_async_en(path: Instance) -> Self {
        RegisterAny::AsyncEn(AsyncDffEnable::new(path))
    }

    pub fn new_sync_en(path: Instance) -> Self {
        RegisterAny::SyncEn(SyncDffEnable::new(path))
    }

    pub fn new_async_mux(path: Instance) -> Self {
        RegisterAny::AsyncMux(AsyncDffMuxEnable::new(path))
    }

    pub fn new_sync_mux(path: Instance) -> Self {
        RegisterAny::SyncMux(SyncDffMuxEnable::new(path))
    }
}
