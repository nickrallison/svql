use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Type, parse2, punctuated::Punctuated};

#[derive(Clone)]
pub struct SubPattern {
    pub field_name: Ident,
    pub ty: Type,
}

#[derive(Clone)]
pub struct Connection {
    pub from_sub: Ident,
    pub from_port: Ident,
    pub to_sub: Ident,
    pub to_port: Ident,
}

pub struct Ast {
    pub name: Ident,
    pub subs: Vec<SubPattern>,
    pub connections: Vec<Connection>,
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

        // Parse "subs: [ ... ]"
        let subs_kw: Ident = input.parse()?;
        if subs_kw != "subs" {
            return Err(input.error("expected 'subs'"));
        }
        input.parse::<Token![:]>()?;
        let subs_content;
        syn::bracketed!(subs_content in input);
        let subs_punctuated: Punctuated<SubPattern, Token![,]> =
            subs_content.parse_terminated(SubPattern::parse, Token![,])?;
        let subs = subs_punctuated.into_iter().collect();

        // Optional trailing comma after subs array
        let _ = input.parse::<Token![,]>();

        // Check if there's a connections section
        let connections = if input.peek(Ident) {
            let conn_kw: Ident = input.parse()?;
            if conn_kw != "connections" {
                return Err(input.error("expected 'connections' or end of input"));
            }
            input.parse::<Token![:]>()?;
            let conn_content;
            syn::bracketed!(conn_content in input);
            let conn_punctuated: Punctuated<Connection, Token![,]> =
                conn_content.parse_terminated(Connection::parse, Token![,])?;
            conn_punctuated.into_iter().collect()
        } else {
            Vec::new()
        };

        Ok(Ast {
            name,
            subs,
            connections,
        })
    }
}

impl Parse for SubPattern {
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        Ok(SubPattern { field_name, ty })
    }
}

impl Parse for Connection {
    fn parse(input: ParseStream) -> Result<Self> {
        let from_sub = input.parse::<Ident>()?;
        input.parse::<Token![.]>()?;
        let from_port = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        let to_sub = input.parse::<Ident>()?;
        input.parse::<Token![.]>()?;
        let to_port = input.parse::<Ident>()?;
        Ok(Connection {
            from_sub,
            from_port,
            to_sub,
            to_port,
        })
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
    fn valid_syntax_with_connections() {
        parse(quote! {
            name: TestComposite,
            subs: [
                field1: Type1,
                field2: Type2
            ],
            connections: [
                field1 . out => field2 . inp
            ]
        });
    }

    #[test]
    fn valid_syntax_without_connections() {
        parse(quote! {
            name: TestComposite,
            subs: [
                field1: Type1,
                field2: Type2
            ]
        });
    }
}
