use crate::{Match, Search, State, Wire, WithPath, instance::Instance, netlist::SearchableNetlist};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_macros::netlist;

// ============================================================================
// Pattern Definitions using netlist! macro
// ============================================================================

netlist! {
    name: AsyncDffEnable,
    module_name: "async_dff_enable",
    file: "examples/patterns/security/locked_reg/verilog/async_dff_enable.v",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
}

netlist! {
    name: SyncDffEnable,
    module_name: "sync_dff_enable",
    file: "examples/patterns/security/locked_reg/verilog/sync_dff_enable.v",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
}

netlist! {
    name: DffMuxEnable,
    module_name: "dff_mux_enable",
    file: "examples/patterns/security/locked_reg/verilog/dff_mux_enable.v",
    inputs: [clk, d, resetn, enable],
    outputs: [q]
}

// ============================================================================
// Enum Composite for Any Register Type
// ============================================================================

/// Represents any type of enabled register that could be used as a locked register
///
/// Each variant is a different flip-flop pattern with an explicit enable signal
/// that should be controlled by unlock logic to prevent unauthorized writes.
#[derive(Debug, Clone)]
pub enum RegisterAny<S>
where
    S: State,
{
    AsyncEnable(AsyncDffEnable<S>),
    SyncEnable(SyncDffEnable<S>),
    MuxEnable(DffMuxEnable<S>),
}

impl<S> WithPath<S> for RegisterAny<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        match self {
            RegisterAny::AsyncEnable(inner) => inner.find_port(p),
            RegisterAny::SyncEnable(inner) => inner.find_port(p),
            RegisterAny::MuxEnable(inner) => inner.find_port(p),
        }
    }

    fn path(&self) -> Instance {
        match self {
            RegisterAny::AsyncEnable(inner) => inner.path(),
            RegisterAny::SyncEnable(inner) => inner.path(),
            RegisterAny::MuxEnable(inner) => inner.path(),
        }
    }
}

impl RegisterAny<Search> {
    pub fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        // Merge contexts from all pattern types
        let async_ctx = AsyncDffEnable::<Search>::context(driver, config)?;
        let sync_ctx = SyncDffEnable::<Search>::context(driver, config)?;
        let mux_ctx = DffMuxEnable::<Search>::context(driver, config)?;

        Ok(async_ctx.merge(sync_ctx).merge(mux_ctx))
    }

    pub fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<RegisterAny<Match<'ctx>>> {
        use crate::netlist::SearchableNetlist;

        tracing::info!("RegisterAny::query: searching for enabled register patterns");

        // Query all pattern types
        let async_matches = AsyncDffEnable::<Search>::query(
            haystack_key,
            context,
            path.child("async_enable".to_string()),
            config,
        );

        let sync_matches = SyncDffEnable::<Search>::query(
            haystack_key,
            context,
            path.child("sync_enable".to_string()),
            config,
        );

        let mux_matches = DffMuxEnable::<Search>::query(
            haystack_key,
            context,
            path.child("mux_enable".to_string()),
            config,
        );

        tracing::info!(
            "RegisterAny::query: Found {} async, {} sync, {} mux patterns",
            async_matches.len(),
            sync_matches.len(),
            mux_matches.len()
        );

        // Collect all matches into enum variants
        let mut all_matches = Vec::new();

        all_matches.extend(async_matches.into_iter().map(RegisterAny::AsyncEnable));

        all_matches.extend(sync_matches.into_iter().map(RegisterAny::SyncEnable));

        all_matches.extend(mux_matches.into_iter().map(RegisterAny::MuxEnable));

        tracing::info!(
            "RegisterAny::query: Total {} enabled registers found",
            all_matches.len()
        );

        all_matches
    }
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
            RegisterAny::AsyncEnable(dff) => &dff.enable,
            RegisterAny::SyncEnable(dff) => &dff.enable,
            RegisterAny::MuxEnable(dff) => &dff.enable,
        }
    }

    /// Get a description of the register type for reporting
    pub fn register_type(&self) -> String {
        match self {
            RegisterAny::AsyncEnable(_) => "AsyncDffEnable".to_string(),
            RegisterAny::SyncEnable(_) => "SyncDffEnable".to_string(),
            RegisterAny::MuxEnable(_) => "DffMuxEnable".to_string(),
        }
    }
}
