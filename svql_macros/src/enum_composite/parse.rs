use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{
    Ident, LitStr, Result, Token, braced, bracketed, parenthesized, parse2, punctuated::Punctuated,
};

#[derive(Clone)]
pub struct Variant {
    pub variant_name: Ident,
    pub inst_name: LitStr,
    pub ty: syn::Type,
}

#[derive(Clone)]
pub struct CommonPort {
    pub field_name: Ident,
    pub method_name: LitStr,
}

pub struct Ast {
    pub name: Ident,
    pub variants: Vec<Variant>,
    pub common_ports: Vec<CommonPort>,
}

impl Parse for Ast {
    fn parse(input: ParseStream) -> Result<Self> {
        let name_kw: Ident = input.parse()?;
        if name_kw != "name" {
            return Err(input.error("expected 'name'"));
        }
        input.parse::<Token![:]>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;

        let variants_kw: Ident = input.parse()?;
        if variants_kw != "variants" {
            return Err(input.error("expected 'variants'"));
        }
        input.parse::<Token![:]>()?;
        let variants_content;
        bracketed!(variants_content in input);
        let variants_punctuated: Punctuated<Variant, Token![,]> =
            variants_content.parse_terminated(Variant::parse, Token![,])?;
        let variants = variants_punctuated.into_iter().collect();

        let _ = input.parse::<Token![,]>();

        let common_ports = if !input.is_empty() && input.peek(Ident) {
            let ports_kw: Ident = input.parse()?;
            if ports_kw != "common_ports" {
                return Err(input.error("expected 'common_ports' or end of input"));
            }
            input.parse::<Token![:]>()?;

            let ports_content;
            braced!(ports_content in input);

            let ports_punctuated: Punctuated<CommonPort, Token![,]> =
                ports_content.parse_terminated(CommonPort::parse, Token![,])?;
            ports_punctuated.into_iter().collect()
        } else {
            Vec::new()
        };

        Ok(Ast {
            name,
            variants,
            common_ports,
        })
    }
}

impl Parse for Variant {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let variant_name = content.parse::<Ident>()?;
        content.parse::<Token![,]>()?;
        let inst_name = content.parse::<LitStr>()?;
        content.parse::<Token![,]>()?;
        let ty = content.parse::<syn::Type>()?;
        Ok(Variant {
            variant_name,
            inst_name,
            ty,
        })
    }
}

impl Parse for CommonPort {
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let method_name = input.parse::<LitStr>()?;
        Ok(CommonPort {
            field_name,
            method_name,
        })
    }
}

pub fn parse(ts: TokenStream) -> Ast {
    match parse2::<Ast>(ts) {
        Ok(ast) => ast,
        Err(e) => abort!(e.span(), e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_common_ports_single() {
        let ts = quote! {
            name: TestEnum,
            variants: [
                (VarA, "a", TypeA),
                (VarB, "b", TypeB)
            ],
            common_ports: {
                clk: "clock"
            }
        };
        let ast = parse(ts);
        assert_eq!(ast.common_ports.len(), 1);
        assert_eq!(ast.common_ports[0].field_name.to_string(), "clk");
        assert_eq!(ast.common_ports[0].method_name.value(), "clock");
    }

    #[test]
    fn parse_common_ports_multiple() {
        let ts = quote! {
            name: DffEnum,
            variants: [(A, "a", T)],
            common_ports: {
                clk: "clock",
                d: "data_input",
                q: "output"
            }
        };
        let ast = parse(ts);
        assert_eq!(ast.common_ports.len(), 3);
        assert_eq!(ast.common_ports[2].method_name.value(), "output");
    }

    #[test]
    fn parse_common_ports_none() {
        let ts = quote! {
            name: NoCommon,
            variants: [(A, "a", T)]
        };
        let ast = parse(ts);
        assert!(ast.common_ports.is_empty());
    }

    #[test]
    fn parse_common_ports_trailing_comma() {
        let ts = quote! {
            name: Trailing,
            variants: [(A, "a", T)],
            common_ports: {
                x: "get_x",
            }
        };
        let ast = parse(ts);
        assert_eq!(ast.common_ports.len(), 1);
    }
}
