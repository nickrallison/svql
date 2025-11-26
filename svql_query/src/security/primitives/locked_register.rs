use crate::{State, Wire};
use svql_macros::{netlist, variant};

// ============================================================================
// Pattern Definitions using netlist! macro
// ============================================================================

#[netlist(
    file = "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
    name = "async_en"
)]
pub struct AsyncDffEnable<S: State> {
    pub clk: Wire<S>,
    pub data_in: Wire<S>,
    pub resetn: Wire<S>,
    pub write_en: Wire<S>,
    pub data_out: Wire<S>,
}

#[netlist(
    file = "examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il",
    name = "sync_en"
)]
pub struct SyncDffEnable<S: State> {
    pub clk: Wire<S>,
    pub data_in: Wire<S>,
    pub resetn: Wire<S>,
    pub write_en: Wire<S>,
    pub data_out: Wire<S>,
}

#[netlist(
    file = "examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il",
    name = "sync_mux"
)]
pub struct SyncDffMuxEnable<S: State> {
    pub clk: Wire<S>,
    pub data_in: Wire<S>,
    pub resetn: Wire<S>,
    pub write_en: Wire<S>,
    pub data_out: Wire<S>,
}

#[netlist(
    file = "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
    name = "async_mux"
)]
pub struct AsyncDffMuxEnable<S: State> {
    pub clk: Wire<S>,
    pub data_in: Wire<S>,
    pub resetn: Wire<S>,
    pub write_en: Wire<S>,
    pub data_out: Wire<S>,
}

#[variant(ports(clk, data_in, data_out, resetn, write_en))]
pub enum LockedRegister<S: State> {
    #[variant(map(
        clk = "clk",
        data_in = "data_in",
        data_out = "data_out",
        resetn = "resetn",
        write_en = "write_en"
    ))]
    AsyncEn(AsyncDffEnable<S>),

    #[variant(map(
        clk = "clk",
        data_in = "data_in",
        data_out = "data_out",
        resetn = "resetn",
        write_en = "write_en"
    ))]
    SyncEn(SyncDffEnable<S>),

    #[variant(map(
        clk = "clk",
        data_in = "data_in",
        data_out = "data_out",
        resetn = "resetn",
        write_en = "write_en"
    ))]
    AsyncMux(AsyncDffMuxEnable<S>),

    #[variant(map(
        clk = "clk",
        data_in = "data_in",
        data_out = "data_out",
        resetn = "resetn",
        write_en = "write_en"
    ))]
    SyncMux(SyncDffMuxEnable<S>),
}

impl<S: State> LockedRegister<S> {
    /// Get the enable wire for connection validation
    pub fn enable_wire(&self) -> Option<&Wire<S>> {
        self.write_en()
    }

    /// Get a description of the register type for reporting
    pub fn register_type(&self) -> String {
        match self {
            LockedRegister::AsyncEn(_) => "AsyncDffEnable".to_string(),
            LockedRegister::SyncEn(_) => "SyncDffEnable".to_string(),
            LockedRegister::AsyncMux(_) => "AsyncDffMuxEnable".to_string(),
            LockedRegister::SyncMux(_) => "SyncDffMuxEnable".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}
