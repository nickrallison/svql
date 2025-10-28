use super::parse::{Ast, Connection, SubPattern};

pub struct Model {
    pub name: syn::Ident,
    pub subs: Vec<SubPattern>,
    pub connections: Vec<Vec<Connection>>,
}

pub fn analyze(ast: Ast) -> Model {
    // For now, just pass through. Could add validation here later.
    Model {
        name: ast.name,
        subs: ast.subs,
        connections: ast.connections,
    }
}
