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
    use super::*;
    use itertools::iproduct;
    use proc_macro2::TokenStream;
    use quote::quote; // For higher-order combinations
    use syn::{Ident, parse2};

    fn try_parse(ts: TokenStream) -> Result<Ast> {
        parse2::<Ast>(ts)
    }

    // Existing success tests (unchanged; they pass)
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

    // Existing + new success tests (unchanged; they pass)
    #[test]
    fn valid_syntax_trailing_commas() {
        let ast = parse(quote! {
            name: TrailingNetlist,
            module_name: "trailing",
            file: "path/to/trailing.v",
            inputs: [a, b,],
            outputs: [y,]
        });
        assert_eq!(ast.inputs.len(), 2);
        assert_eq!(ast.outputs.len(), 1);
    }

    #[test]
    fn valid_syntax_non_literal_expr() {
        // Parse succeeds (analyze panics later on non-lits; this tests parse isolation)
        let ast = parse(quote! {
            name: ExprNetlist,
            module_name: some_var,  // Not a lit
            file: concat!("path", "/to/file.v"),  // Expr
            inputs: [in1],
            outputs: [out1]
        });
        assert_eq!(ast.name.to_string(), "ExprNetlist");
        assert_eq!(ast.inputs.len(), 1);
    }

    #[test]
    fn full_ast_multiple_ports() {
        let ast = parse(quote! {
            name: MultiPortNetlist,
            module_name: "multi",
            file: "path/to/multi.v",
            inputs: [clk, data, reset],
            outputs: [q, valid]
        });
        assert_eq!(ast.name.to_string(), "MultiPortNetlist");
        assert_eq!(ast.inputs.len(), 3);
        assert_eq!(ast.outputs.len(), 2);
        assert_eq!(ast.inputs[0].name.to_string(), "clk");
        assert_eq!(ast.outputs[1].name.to_string(), "valid");
    }

    // NEW: Error tests using try_parse (avoids proc-macro-error)
    #[test]
    fn error_missing_comma_after_name() {
        let res = try_parse(quote! {
            name: BadSections
            module_name: "bad",  // Missing ,
            file: "path.v",
            inputs: [],
            outputs: []
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected `,`"),
            "Expected comma error, got: {}",
            err
        );
    }

    #[test]
    fn error_invalid_port_name() {
        let res = try_parse(quote! {
            name: BadPort,
            module_name: "bad",
            file: "path.v",
            inputs: [fn],  // 'fn' is a keyword (invalid ident)
            outputs: []
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected ident")
                || err.to_string().contains("unexpected token"),
            "Expected invalid ident error, got: {}",
            err
        ); // Syn may say "expected ident" or token error
    }

    // FIXED: Generated test - only valid paths (higher-order over success cases)
    #[test]
    fn generated_port_tests() {
        // Define valid port lists for inputs/outputs
        let valid_input_ports = vec![
            vec!["in1"],      // Single
            vec!["clk", "d"], // Multiple
            vec![],           // Empty
        ];
        let valid_output_ports = vec![vec!["out1"], vec!["q", "valid"], vec![]];

        // Higher-order: Product of valid input/output combos
        iproduct!(valid_input_ports, valid_output_ports).for_each(|(inputs, outputs)| {
            let input_idents: Vec<TokenStream> = inputs
                .iter()
                .map(|p| {
                    let ident = Ident::new(p, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            let output_idents: Vec<TokenStream> = outputs
                .iter()
                .map(|p| {
                    let ident = Ident::new(p, proc_macro2::Span::call_site());
                    quote! { #ident }
                })
                .collect();
            let full = quote! {
                name: PortTest,
                module_name: "test",
                file: "path.v",
                inputs: [#(#input_idents),*],
                outputs: [#(#output_idents),*]
            };
            let ast = parse(full);
            assert_eq!(
                ast.inputs.len() as usize,
                inputs.len(),
                "Input ports mismatch"
            );
            assert_eq!(
                ast.outputs.len() as usize,
                outputs.len(),
                "Output ports mismatch"
            );
        });
    }

    // NEW: Dedicated error subtest for generated cases (e.g., invalid port in list)
    #[test]
    fn generated_port_error_test() {
        let res = try_parse(quote! {
            name: PortErrorTest,
            module_name: "error",
            file: "path.v",
            inputs: [123],  // Invalid: not an ident
            outputs: []
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected ident"),
            "Expected invalid port error"
        );
    }
}
