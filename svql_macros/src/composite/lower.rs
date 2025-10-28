use super::analyze::Model;
use super::parse::{Connection, SubPattern};

pub struct Ir {
    pub name: syn::Ident,
    pub subs: Vec<SubPattern>,
    pub connections: Vec<Connection>,
}

pub fn lower(model: Model) -> Ir {
    Ir {
        name: model.name,
        subs: model.subs,
        connections: model.connections,
    }
}
