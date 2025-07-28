use proc_macro_error::abort;
use svql_common::pattern::ffi::Pattern;
use svql_pat::extract_pattern;

use super::parse::Ast;

pub struct Model {
    pub vis: syn::Visibility,
    pub iface_ident: syn::Ident,
    pub file_path: String,
    pub module_name: String,
    pub pattern: Pattern,
}

pub fn analyze(ast: Ast) -> Model {
    let pattern = extract_pattern(
        &ast.file_path,
        &ast.module_name,
        Some(&ast.yosys),
        Some(&ast.svql_pat_plugin_path),
    )
    .unwrap_or_else(|e| abort!(ast.iface_ident, "extract_pattern failed: {}", e));

    Model {
        vis: ast.vis,
        iface_ident: ast.iface_ident,
        file_path: ast.file_path,
        module_name: ast.module_name,
        pattern,
    }
}
