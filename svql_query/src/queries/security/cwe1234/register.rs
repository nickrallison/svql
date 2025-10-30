use crate::{
    Match, Search, State, Wire, WithPath,
    instance::Instance,
    netlist::{self, SearchableNetlist},
};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_macros::{enum_composite, netlist};

// ============================================================================
// Pattern Definitions using netlist! macro
// ============================================================================

netlist! {
    name: AsyncDffEnable,
    module_name: "async_en",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/async_en.il",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
}

netlist! {
    name: SyncDffEnable,
    module_name: "sync_en",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
}

netlist! {
    name: SyncDffMuxEnable,
    module_name: "sync_mux",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
}

netlist! {
    name: AsyncDffMuxEnable,
    module_name: "async_mux",
    file: "examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
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
            RegisterAny::AsyncEn(dff) => &dff.enable,
            RegisterAny::SyncEn(dff) => &dff.enable,
            RegisterAny::AsyncMux(dff) => &dff.enable,
            RegisterAny::SyncMux(dff) => &dff.enable,
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

// ============================================================================
// Enum Composite for Any Register Type
// ============================================================================

// Represents any type of enabled register that could be used as a locked register
//
// Each variant is a different flip-flop pattern with an explicit enable signal
// that should be controlled by unlock logic to prevent unauthorized writes.
// #[derive(Debug, Clone)]
// pub enum RegisterAny<S>
// where
//     S: State,
// {
//     AsyncEnable(AsyncDffEnable<S>),
//     SyncEnable(SyncDffEnable<S>),
//     AsyncMuxEnable(AsyncDffMuxEnable<S>),
//     SyncMuxEnable(SyncDffMuxEnable<S>),
// }

// impl<S> WithPath<S> for RegisterAny<S>
// where
//     S: State,
// {
//     fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
//         match self {
//             RegisterAny::AsyncEnable(inner) => inner.find_port(p),
//             RegisterAny::SyncEnable(inner) => inner.find_port(p),
//             RegisterAny::AsyncMuxEnable(inner) => inner.find_port(p),
//             RegisterAny::SyncMuxEnable(inner) => inner.find_port(p),
//         }
//     }

//     fn path(&self) -> Instance {
//         match self {
//             RegisterAny::AsyncEnable(inner) => inner.path(),
//             RegisterAny::SyncEnable(inner) => inner.path(),
//             RegisterAny::AsyncMuxEnable(inner) => inner.path(),
//             RegisterAny::SyncMuxEnable(inner) => inner.path(),
//         }
//     }
// }

// impl RegisterAny<Search> {
//     pub fn context(
//         driver: &Driver,
//         config: &ModuleConfig,
//     ) -> Result<Context, Box<dyn std::error::Error>> {
//         // Merge contexts from all pattern types
//         let async_en_ctx = AsyncDffEnable::<Search>::context(driver, config)?;
//         let sync_en_ctx = SyncDffEnable::<Search>::context(driver, config)?;
//         let async_mux_ctx = AsyncDffMuxEnable::<Search>::context(driver, config)?;
//         let sync_mux_ctx = SyncDffMuxEnable::<Search>::context(driver, config)?;

//         Ok(async_en_ctx
//             .merge(sync_en_ctx)
//             .merge(async_mux_ctx)
//             .merge(sync_mux_ctx))
//     }

//     pub fn query<'ctx>(
//         haystack_key: &DriverKey,
//         context: &'ctx Context,
//         path: Instance,
//         config: &Config,
//     ) -> Vec<RegisterAny<Match<'ctx>>> {
//         use crate::netlist::SearchableNetlist;

//         tracing::info!("RegisterAny::query: searching for enabled register patterns");

//         // Query all pattern types
//         let async_en_matches = AsyncDffEnable::<Search>::query(
//             haystack_key,
//             context,
//             path.child("async_enable".to_string()),
//             config,
//         );

//         let sync_en_matches = SyncDffEnable::<Search>::query(
//             haystack_key,
//             context,
//             path.child("sync_enable".to_string()),
//             config,
//         );

//         let async_mux_matches = AsyncDffMuxEnable::<Search>::query(
//             haystack_key,
//             context,
//             path.child("async_mux_enable".to_string()),
//             config,
//         );

//         let sync_mux_matches = SyncDffMuxEnable::<Search>::query(
//             haystack_key,
//             context,
//             path.child("sync_mux_enable".to_string()),
//             config,
//         );

//         tracing::info!(
//             "RegisterAny::query: Found {} async_en, {} sync_en, {} async_mux, {} sync_mux patterns",
//             async_en_matches.len(),
//             sync_en_matches.len(),
//             async_mux_matches.len(),
//             sync_mux_matches.len()
//         );

//         // Collect all matches into enum variants
//         let mut all_matches = Vec::new();

//         all_matches.extend(async_matches.into_iter().map(RegisterAny::AsyncEnable));

//         all_matches.extend(sync_matches.into_iter().map(RegisterAny::SyncEnable));

//         all_matches.extend(mux_matches.into_iter().map(RegisterAny::MuxEnable));

//         tracing::info!(
//             "RegisterAny::query: Total {} enabled registers found",
//             all_matches.len()
//         );

//         all_matches
//     }
// }

// // Helper methods for RegisterAny<Match>
// impl<S> RegisterAny<S>
// where
//     S: State,
// {
//     /// Get the enable wire for connection validation
//     /// This is what should connect to the unlock logic output
//     pub fn enable_wire(&self) -> &Wire<S> {
//         match self {
//             RegisterAny::AsyncEnable(dff) => &dff.enable,
//             RegisterAny::SyncEnable(dff) => &dff.enable,
//             RegisterAny::MuxEnable(dff) => &dff.enable,
//         }
//     }

//     /// Get a description of the register type for reporting
//     pub fn register_type(&self) -> String {
//         match self {
//             RegisterAny::AsyncEnable(_) => "AsyncDffEnable".to_string(),
//             RegisterAny::SyncEnable(_) => "SyncDffEnable".to_string(),
//             RegisterAny::MuxEnable(_) => "DffMuxEnable".to_string(),
//         }
//     }
// }
