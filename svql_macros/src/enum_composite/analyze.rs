use super::parse::{Ast, Variant};

pub struct Model {
    pub name: syn::Ident,
    pub variants: Vec<Variant>,
}

pub fn analyze(ast: Ast) -> Model {
    // For now, just pass through. Could add validation here later (e.g., unique inst_names).
    Model {
        name: ast.name,
        variants: ast.variants,
    }
}
