use super::parse::{Ast, Port};

pub struct Model {
    pub name: syn::Ident,
    pub module_name: String,
    pub file_path: String,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

pub fn analyze(ast: Ast) -> Model {
    // Extract string literals from exprs (assuming they are string literals as per usage)
    let module_name = match &ast.module_name {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => s.value(),
        _ => panic!("module_name must be a string literal"),
    };
    let file_path = match &ast.file {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => s.value(),
        _ => panic!("file must be a string literal"),
    };

    Model {
        name: ast.name,
        module_name,
        file_path,
        inputs: ast.inputs,
        outputs: ast.outputs,
    }
}
