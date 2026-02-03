use svql_query::prelude::*;

// 1. Define the primitive DFF with enable
svql_query::define_dff_primitive!(
    LockedRegEn,
    [
        (clk, input),
        (data_in, input),
        (data_out, output),
        (resetn, input),
        (write_en, input)
    ],
    |cell| {
        if let prjunnamed_netlist::Cell::Dff(ff) = cell {
            ff.has_enable()
        } else {
            false
        }
    }
);

// 2. Define the Netlist-based implementations (DFF + Mux logic)
#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il",
    module = "sync_mux"
)]
pub struct SyncDffMuxEnable {
    #[port(input)]
    pub clk: Wire,
    #[port(input)]
    pub data_in: Wire,
    #[port(input)]
    pub resetn: Wire,
    #[port(input)]
    pub write_en: Wire,
    #[port(output)]
    pub data_out: Wire,
}

#[derive(Debug, Clone, Netlist)]
#[netlist(
    file = "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
    module = "async_mux"
)]
pub struct AsyncDffMuxEnable {
    #[port(input)]
    pub clk: Wire,
    #[port(input)]
    pub data_in: Wire,
    #[port(input)]
    pub resetn: Wire,
    #[port(input)]
    pub write_en: Wire,
    #[port(output)]
    pub data_out: Wire,
}

// 3. Define the Variant that unifies all three implementations
#[derive(Debug, Clone, Variant)]
#[variant_ports(
    input(clk),
    input(data_in),
    output(data_out),
    input(resetn),
    input(write_en)
)]
pub enum LockedRegister {
    #[map(
        clk = ["clk"],
        data_in = ["data_in"],
        data_out = ["data_out"],
        resetn = ["resetn"],
        write_en = ["write_en"]
    )]
    En(LockedRegEn),

    #[map(
        clk = ["clk"],
        data_in = ["data_in"],
        data_out = ["data_out"],
        resetn = ["resetn"],
        write_en = ["write_en"]
    )]
    AsyncMux(AsyncDffMuxEnable),

    #[map(
        clk = ["clk"],
        data_in = ["data_in"],
        data_out = ["data_out"],
        resetn = ["resetn"],
        write_en = ["write_en"]
    )]
    SyncMux(SyncDffMuxEnable),
}

impl LockedRegister {
    /// Helper to access the write enable wire regardless of the underlying variant
    pub fn write_en(&self) -> Wire {
        match self {
            Self::En(inner) => inner.write_en.clone(),
            Self::AsyncMux(inner) => inner.write_en.clone(),
            Self::SyncMux(inner) => inner.write_en.clone(),
        }
    }
}
