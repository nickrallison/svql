use crate::{impl_dff_primitive, prelude::*};
use svql_macros::{netlist, variant};

impl_dff_primitive!(
    LockedRegEn,
    [clk, data_in, data_out, resetn, write_en],
    |ff| ff.has_enable(),
    "Matches primitive DFFs with an enable."
);

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
    En(LockedRegEn<S>),

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
    pub fn enable_wire(&self) -> Option<&Wire<S>> {
        self.write_en()
    }
}
