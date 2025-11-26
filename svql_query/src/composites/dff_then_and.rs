use crate::State;
use crate::instance::Instance;
use crate::primitives::and::AndGate;
use crate::primitives::dff::Sdffe;
use crate::traits::{ConnectionBuilder, Topology};
use svql_macros::composite;

#[composite]
pub struct SdffeThenAnd<S: State> {
    #[path]
    pub path: Instance,

    #[submodule]
    pub sdffe: Sdffe<S>,

    #[submodule]
    pub and_gate: AndGate<S>,
}

impl<S: State> Topology<S> for SdffeThenAnd<S> {
    fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
        // Connect Q to A
        ctx.connect(Some(&self.sdffe.q), Some(&self.and_gate.a));
        // Connect Q to B
        ctx.connect(Some(&self.sdffe.q), Some(&self.and_gate.b));
    }
}
