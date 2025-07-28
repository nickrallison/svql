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
    let file_path = std::path::PathBuf::from(&ast.file_path);
    let yosys_path = std::path::PathBuf::from(&ast.yosys);
    let plugin_library_path = std::path::PathBuf::from(&ast.svql_pat_plugin_path);
    let pattern = extract_pattern(
        file_path,
        ast.module_name.clone(),
        Some(yosys_path),
        Some(plugin_library_path),
    )
    .unwrap_or_else(|e| abort!(ast.iface_ident, "svql-query: extract_pattern failed: {}", e));

    Model {
        vis: ast.vis,
        iface_ident: ast.iface_ident,
        file_path: ast.file_path,
        module_name: ast.module_name,
        pattern,
    }
}
