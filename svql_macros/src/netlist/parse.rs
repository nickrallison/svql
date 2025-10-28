use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Result, Token, bracketed, parse2, punctuated::Punctuated};

#[derive(Clone)]
pub struct Port {
    pub name: Ident,
}

pub struct Ast {
    pub name: Ident,
    pub module_name: Expr,
    pub file: Expr,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse "name: Identifier"
        let name_kw: Ident = input.parse()?;
        if name_kw != "name" {
            return Err(input.error("expected 'name'"));
        }
        input.parse::<Token![:]>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        // Parse "module_name: expr"
        let module_kw: Ident = input.parse()?;
        if module_kw != "module_name" {
            return Err(input.error("expected 'module_name'"));
        }
        input.parse::<Token![:]>()?;
        let module_name = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;

        // Parse "file: expr"
        let file_kw: Ident = input.parse()?;
        if file_kw != "file" {
            return Err(input.error("expected 'file'"));
        }
        input.parse::<Token![:]>()?;
        let file = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;

        // Parse "inputs: [ ... ]"
        let inputs_kw: Ident = input.parse()?;
        if inputs_kw != "inputs" {
            return Err(input.error("expected 'inputs'"));
        }
        input.parse::<Token![:]>()?;
        let inputs_content;
        bracketed!(inputs_content in input);
        let inputs_punctuated: Punctuated<Port, Token![,]> =
            inputs_content.parse_terminated(Port::parse, Token![,])?;
        let inputs = inputs_punctuated.into_iter().collect();

        // Optional trailing comma after inputs array
        let _ = input.parse::<Token![,]>();

        // Parse "outputs: [ ... ]"
        let outputs_kw: Ident = input.parse()?;
        if outputs_kw != "outputs" {
            return Err(input.error("expected 'outputs'"));
        }
        input.parse::<Token![:]>()?;
        let outputs_content;
        bracketed!(outputs_content in input);
        let outputs_punctuated: Punctuated<Port, Token![,]> =
            outputs_content.parse_terminated(Port::parse, Token![,])?;
        let outputs = outputs_punctuated.into_iter().collect();

        Ok(Ast {
            name,
            module_name,
            file,
            inputs,
            outputs,
        })
    }
}

impl Parse for Port {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        Ok(Port { name })
    }
}

pub fn parse(ts: TokenStream) -> Ast {
    match parse2::<Ast>(ts) {
        Ok(ast) => ast,
        Err(e) => {
            abort!(e.span(), e)
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn valid_syntax() {
        parse(quote! {
            name: AndGate,
            module_name: "and_gate",
            file: "examples/patterns/basic/and/verilog/and_gate.v",
            inputs: [a, b],
            outputs: [y]
        });
    }

    #[test]
    fn valid_syntax_empty_lists() {
        parse(quote! {
            name: EmptyNetlist,
            module_name: "empty",
            file: "path/to/empty.v",
            inputs: [],
            outputs: []
        });
    }
}
