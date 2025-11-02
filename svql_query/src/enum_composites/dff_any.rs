// svql_query/src/enum_composites/dff_any.rs
use crate::primitives::dff::{
    AsyncResetDff, AsyncResetEnableDff, EnableDff, SimpleDff, SyncResetDff, SyncResetEnableDff,
};
use crate::{Match, State, Wire};
use svql_macros::enum_composite;

enum_composite! {
    name: DffAny,
    variants: [
        (Simple, "simple_dff", SimpleDff),
        (SyncReset, "sync_reset_dff", SyncResetDff),
        (AsyncReset, "async_reset_dff", AsyncResetDff),
        (SyncResetEnable, "sync_reset_enable_dff", SyncResetEnableDff),
        (AsyncResetEnable, "async_reset_enable_dff", AsyncResetEnableDff),
        (Enable, "enable_dff", EnableDff)
    ],
    common_ports: {
        clk: "clock",
        d: "data_input",
        q: "output"
    }
}

impl<S> DffAny<S>
where
    S: State,
{
    pub fn dff_type(&self) -> &'static str {
        match self {
            DffAny::Simple(_) => "Simple DFF",
            DffAny::SyncReset(_) => "Sync Reset DFF",
            DffAny::AsyncReset(_) => "Async Reset DFF",
            DffAny::SyncResetEnable(_) => "Sync Reset+Enable DFF",
            DffAny::AsyncResetEnable(_) => "Async Reset+Enable DFF",
            DffAny::Enable(_) => "Enable DFF",
        }
    }

    pub fn has_reset(&self) -> bool {
        matches!(
            self,
            DffAny::SyncReset(_)
                | DffAny::AsyncReset(_)
                | DffAny::SyncResetEnable(_)
                | DffAny::AsyncResetEnable(_)
        )
    }

    pub fn has_enable(&self) -> bool {
        matches!(
            self,
            DffAny::SyncResetEnable(_) | DffAny::AsyncResetEnable(_) | DffAny::Enable(_)
        )
    }

    pub fn reset_wire(&self) -> Option<&Wire<S>> {
        match self {
            DffAny::SyncReset(dff) => Some(&dff.srst),
            DffAny::AsyncReset(dff) => Some(&dff.arst),
            DffAny::SyncResetEnable(dff) => Some(&dff.srst),
            DffAny::AsyncResetEnable(dff) => Some(&dff.arst),
            _ => None,
        }
    }

    pub fn enable_wire(&self) -> Option<&Wire<S>> {
        match self {
            DffAny::SyncResetEnable(dff) => Some(&dff.en),
            DffAny::AsyncResetEnable(dff) => Some(&dff.en),
            DffAny::Enable(dff) => Some(&dff.en),
            _ => None,
        }
    }
}
