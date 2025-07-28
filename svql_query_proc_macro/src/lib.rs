use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod module;
use crate::module::analyze;
use crate::module::codegen;
use crate::module::lower;
use crate::module::parse;

/// Attribute style macro:
/// #[module(file = "...", module = "...", yosys = "...", svql_pat_plugin_path = "...")]  pub struct MyIfc;
#[proc_macro_attribute]
#[proc_macro_error]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse::parse(attr.into(), item.into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    let ts = codegen::codegen(ir);
    ts.into()
}
