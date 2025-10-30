use proc_macro_error::abort;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token, Type, bracketed, parse2, punctuated::Punctuated};

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
    pub connections: Vec<Vec<Connection>>,
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
        bracketed!(subs_content in input);
        let subs_punctuated: Punctuated<SubPattern, Token![,]> =
            subs_content.parse_terminated(SubPattern::parse, Token![,])?;
        let subs = subs_punctuated.into_iter().collect();

        // Optional trailing comma after subs array
        let _ = input.parse::<Token![,]>();

        // Parse "connections: [ ... ]" with nested groups
        let connections = if input.peek(Ident) {
            let conn_kw: Ident = input.parse()?;
            if conn_kw != "connections" {
                return Err(input.error("expected 'connections' or end of input"));
            }
            input.parse::<Token![:]>()?;
            let connections_content;
            bracketed!(connections_content in input);
            let mut connections = Vec::new();
            while !connections_content.is_empty() {
                let group_content;
                bracketed!(group_content in connections_content);
                let group_punctuated: Punctuated<Connection, Token![,]> =
                    group_content.parse_terminated(Connection::parse, Token![,])?;
                connections.push(group_punctuated.into_iter().collect::<Vec<_>>());

                // Optional comma after group
                if connections_content.peek(Token![,]) {
                    connections_content.parse::<Token![,]>()?;
                }
            }
            connections
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
    use super::*;
    use itertools::iproduct;
    use quote::quote; // For higher-order combinations in generated tests

    fn try_parse(ts: TokenStream) -> Result<Ast> {
        parse2::<Ast>(ts)
    }

    // Existing success tests (unchanged; they pass)
    #[test]
    fn valid_syntax_with_nested_connections() {
        parse(quote! {
            name: TestComposite,
            subs: [
                field1: Type1,
                field2: Type2
            ],
            connections: [
                [
                    field1 . out => field2 . inp,
                    field1 . out2 => field2 . inp2
                ],
                [
                    field3 . out => field4 . inp
                ]
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

    #[test]
    fn valid_syntax_with_single_group() {
        parse(quote! {
            name: TestComposite,
            subs: [
                field1: Type1
            ],
            connections: [
                [
                    field1 . out => field1 . inp
                ]
            ]
        });
    }

    // Existing + new success tests (unchanged; they pass)
    #[test]
    fn valid_syntax_empty_subs() {
        let ast = parse(quote! {
            name: EmptyComposite,
            subs: [],
            connections: []
        });
        assert_eq!(ast.name.to_string(), "EmptyComposite");
        assert!(ast.subs.is_empty());
        assert!(ast.connections.is_empty());
    }

    #[test]
    fn valid_syntax_trailing_commas() {
        let ast = parse(quote! {
            name: TrailingComposite,
            subs: [
                field1: Type1,
                field2: Type2,
            ],
            connections: [
                [
                    field1 . out => field2 . inp,
                ],
            ]
        });
        assert_eq!(ast.subs.len(), 2);
        assert_eq!(ast.connections.len(), 1);
        assert_eq!(ast.connections[0].len(), 1);
    }

    #[test]
    fn valid_syntax_multiple_groups() {
        let ast = parse(quote! {
            name: MultiGroupComposite,
            subs: [
                src: SrcType,
                dst1: DstType1,
                dst2: DstType2
            ],
            connections: [
                [
                    src . out => dst1 . in1
                ],
                [
                    src . out => dst2 . in2,
                    src . out2 => dst2 . in3
                ]
            ]
        });
        assert_eq!(ast.subs.len(), 3);
        assert_eq!(ast.connections.len(), 2);
        assert_eq!(ast.connections[0].len(), 1);
        assert_eq!(ast.connections[1].len(), 2);
    }

    // NEW: Error tests using try_parse (avoids proc-macro-error; asserts on syn::Error)
    #[test]
    fn error_missing_colon_in_sub() {
        let res = try_parse(quote! {
            name: BadSub,
            subs: [
                field1 Type1  // Missing :
            ]
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected `:`"),
            "Expected colon error, got: {}",
            err
        );
    }

    #[test]
    fn error_malformed_connection() {
        let res = try_parse(quote! {
            name: BadConn,
            subs: [src: SrcType, dst: DstType],
            connections: [
                [
                    src . out dst . in1  // Missing =>
                ]
            ]
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected `=>`"),
            "Expected => error, got: {}",
            err
        );
    }

    #[test]
    fn error_invalid_keyword() {
        let res = try_parse(quote! {
            name: Invalid,
            wrong: [  // Not 'subs'
                field1: Type1
            ]
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected ident")
                || err.to_string().contains("expected 'subs'"),
            "Expected keyword error, got: {}",
            err
        ); // Syn may vary: "expected ident" or field mismatch
    }

    // FIXED: Generated test - only valid paths (higher-order over success cases; no panics)
    // Tests combinations of subs/connections without errors
    #[test]
    fn generated_connection_tests() {
        // Define valid subs and connection groups to product over
        let valid_subs = vec![
            quote! { subs: [field1: Type1] },
            quote! { subs: [field1: Type1, field2: Type2] },
        ];
        let valid_conns = vec![
            quote! { connections: [] },                                 // Empty
            quote! { connections: [ [field1 . out => field2 . inp] ] }, // Single group
        ];

        // Higher-order: Product of valid combos, parse each
        iproduct!(valid_subs, valid_conns).for_each(|(subs, conns)| {
            let full = quote! {
                name: VarComposite,
                #subs,
                #conns
            };
            let ast = parse(full);
            assert!(
                !ast.subs.is_empty(),
                "Subs should not be empty in valid case"
            );
            // Additional assertions can be added per combo if needed
        });

        // Explicit full valid parse (as before)
        parse(quote! {
            name: VarComposite,
            subs: [a: TypeA, b: TypeB],
            connections: [
                [a . out => b . in1],
                [a . out2 => b . in2]
            ]
        });
    }

    // NEW: Dedicated error subtest for generated cases (e.g., invalid connection in a group)
    #[test]
    fn generated_connection_error_test() {
        let res = try_parse(quote! {
            name: VarErrorComposite,
            subs: [a: TypeA, b: TypeB],
            connections: [
                [a . out b . in1]  // Malformed (missing =>)
            ]
        });
        let err = match res {
            Ok(_) => panic!("Expected error, but parsing succeeded"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("expected `=>`"),
            "Expected malformed connection error"
        );
    }
}
