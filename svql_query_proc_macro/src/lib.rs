use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod module;
use crate::module::analyze;
use crate::module::codegen;
use crate::module::lower;
use crate::module::parse;

#[proc_macro_derive(Module)]
#[proc_macro_error]
pub fn module(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    let _ = codegen::codegen(ir);
    TokenStream::new()
}
