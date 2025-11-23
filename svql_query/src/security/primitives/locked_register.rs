use crate::{State, Wire, instance::Instance};
use svql_macros::{variant, netlist};

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

variant! {
    name: LockedRegister,
    variants: [
        (AsyncEn, "async_en", AsyncDffEnable),
        (SyncEn, "sync_en", SyncDffEnable),
        (AsyncMux, "async_mux", AsyncDffMuxEnable),
        (SyncMux, "sync_mux", SyncDffMuxEnable)
    ],
    common_ports: {
        clk: "clk",
        data_in: "data_in",
        data_out: "data_out",
        resetn: "resetn",
        write_en: "write_en"
    }
}

// Helper methods for RegisterAny<Match>
impl<S> LockedRegister<S>
where
    S: State,
{
    /// Get the enable wire for connection validation
    /// This is what should connect to the unlock logic output
    pub fn enable_wire(&self) -> &Wire<S> {
        match self {
            LockedRegister::AsyncEn(dff) => &dff.write_en, // FIXED: enable → write_en
            LockedRegister::SyncEn(dff) => &dff.write_en,  // FIXED: enable → write_en
            LockedRegister::AsyncMux(dff) => &dff.write_en, // FIXED: enable → write_en
            LockedRegister::SyncMux(dff) => &dff.write_en, // FIXED: enable → write_en
        }
    }

    /// Get a description of the register type for reporting
    pub fn register_type(&self) -> String {
        match self {
            LockedRegister::AsyncEn(_) => "AsyncDffEnable".to_string(),
            LockedRegister::SyncEn(_) => "SyncDffEnable".to_string(),
            LockedRegister::AsyncMux(_) => "AsyncDffMuxEnable".to_string(),
            LockedRegister::SyncMux(_) => "SyncDffMuxEnable".to_string(),
        }
    }

    pub fn new_async_en(path: Instance) -> Self {
        LockedRegister::AsyncEn(AsyncDffEnable::new(path))
    }

    pub fn new_sync_en(path: Instance) -> Self {
        LockedRegister::SyncEn(SyncDffEnable::new(path))
    }

    pub fn new_async_mux(path: Instance) -> Self {
        LockedRegister::AsyncMux(AsyncDffMuxEnable::new(path))
    }

    pub fn new_sync_mux(path: Instance) -> Self {
        LockedRegister::SyncMux(SyncDffMuxEnable::new(path))
    }
    // NEW: Dummy new for compatibility as composite sub (uses first variant)
    pub fn new(path: Instance) -> Self {
        // Use AsyncEn variant as dummy for search-time construction (via helper)
        // Inner path uses the variant's inst_name
        Self::new_async_en(path.child("async_en".to_string()))
    }
}
