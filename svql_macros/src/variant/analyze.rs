use super::parse::{Ast, CommonPort, Variant};

pub struct Model {
    pub name: syn::Ident,
    pub variants: Vec<Variant>,
    pub common_ports: Vec<CommonPort>,
}

pub fn analyze(ast: Ast) -> Model {
    Model {
        name: ast.name,
        variants: ast.variants,
        common_ports: ast.common_ports,
    }
}
