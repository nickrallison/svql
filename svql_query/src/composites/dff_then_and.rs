use crate::prelude::*;

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
        ctx.connect(Some(&self.sdffe.q), Some(&self.and_gate.a));
        ctx.connect(Some(&self.sdffe.q), Some(&self.and_gate.b));
    }
}
