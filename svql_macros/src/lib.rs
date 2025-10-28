use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod composite;
mod enum_composite;
mod netlist;

#[proc_macro]
#[proc_macro_error]
pub fn composite(input: TokenStream) -> TokenStream {
    let input2 = proc_macro2::TokenStream::from(input);
    let ast = composite::parse::parse(input2);
    let model = composite::analyze::analyze(ast);
    let ir = composite::lower::lower(model);
    let output = composite::codegen::codegen(ir);
    output.into()
}
