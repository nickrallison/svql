// svql_macros/src/composite.rs
#![allow(clippy::too_many_lines, clippy::unnecessary_wraps)]

use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprArray, Fields, Meta, Token, parse::Parse, parse_macro_input,
};

use crate::parsing::{Direction, PathSelector, find_all_attrs, find_attr, parse_nested_paths};

/// A single connection constraint
struct Connection {
    from: PathSelector,
    to: PathSelector,
}

/// An OR group of connections (at least one must be satisfied)
struct OrGroup {
    connections: Vec<Connection>,
}

/// Parsed submodule field
struct SubmoduleField {
    name: syn::Ident,
    ty: syn::Type,
}

/// Parsed alias field
struct AliasField {
    name: syn::Ident,
    direction: Direction,
    target: PathSelector,
}

pub fn composite_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // Ensure it's a struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => abort!(
                input,
                "Composite derive only supports structs with named fields"
            ),
        },
        _ => abort!(input, "Composite derive only supports structs"),
    };

    // Parse struct-level connection attributes
    let or_groups = parse_connection_attrs(&input);

    // Parse submodule fields
    let submodules = parse_submodule_fields(fields);

    // Parse alias fields
    let aliases = parse_alias_fields(fields);

    // Generate implementation
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generate SUBMODULES array
    let submodule_entries: Vec<_> = submodules
        .iter()
        .map(|s| {
            let field_name = s.name.to_string();
            let ty = &s.ty;
            quote! {
                svql_query::session::Submodule::of::<#ty>(#field_name)
            }
        })
        .collect();

    // Generate ALIASES array
    let alias_entries: Vec<_> = aliases
        .iter()
        .map(|a| {
            let port_name = a.name.to_string();
            let target = a.target.to_selector_tokens();
            let constructor = match a.direction {
                Direction::Output => quote! { svql_query::session::Alias::output },
                Direction::Input | Direction::Inout => {
                    quote! { svql_query::session::Alias::input }
                } // Treat as input for now
            };
            quote! {
                #constructor(#port_name, #target)
            }
        })
        .collect();

    // Generate CONNECTIONS in CNF form
    let connection_groups: Vec<_> = or_groups
        .iter()
        .map(|group| {
            let connections: Vec<_> = group
                .connections
                .iter()
                .map(|conn| {
                    let from = conn.from.to_selector_tokens();
                    let to = conn.to.to_selector_tokens();
                    quote! {
                        svql_query::traits::composite::Connection::new(#from, #to)
                    }
                })
                .collect();
            quote! {
                &[#(#connections),*]
            }
        })
        .collect();

    // Generate DEPENDANCIES array
    let dep_entries: Vec<_> = submodules
        .iter()
        .map(|s| {
            let ty = &s.ty;
            quote! {
                <#ty as svql_query::traits::Pattern>::EXEC_INFO
            }
        })
        .collect();

    // Generate rehydrate field assignments for submodules
    let submodule_rehydrate: Vec<_> = submodules
        .iter()
        .map(|s| {
            let field_name = &s.name;
            let field_str = field_name.to_string();
            let ty = &s.ty;
            quote! {
                let #field_name = {
                    let sub_ref = row.sub::<#ty>(#field_str)?;
                    let sub_table = store.get::<#ty>()?;
                    let sub_row = sub_table.row(sub_ref.index())?;
                    #ty::rehydrate(&sub_row, store, driver, key)?
                };
            }
        })
        .collect();

    let alias_rehydrate: Vec<_> = aliases
        .iter()
        .map(|a| {
            let field_name = &a.name;
            let field_str = field_name.to_string();
            quote! {
                let #field_name = row.wire(#field_str)?;
            }
        })
        .collect();

    let submodule_names: Vec<_> = submodules.iter().map(|s| &s.name).collect();
    let alias_names: Vec<_> = aliases.iter().map(|a| &a.name).collect();

    // Generate preload_driver calls
    let preload_calls: Vec<_> = submodules
        .iter()
        .map(|s| {
            let ty = &s.ty;
            quote! {
                <#ty as svql_query::traits::Pattern>::preload_driver(driver, design_key, config)?;
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics svql_query::traits::composite::Composite for #name #ty_generics #where_clause {
            const SUBMODULES: &'static [svql_query::session::Submodule] = &[
                #(#submodule_entries),*
            ];

            const ALIASES: &'static [svql_query::session::Alias] = &[
                #(#alias_entries),*
            ];

            const CONNECTIONS: svql_query::traits::composite::Connections =
                svql_query::traits::composite::Connections {
                    connections: &[#(#connection_groups),*],
                };

            const DEPENDANCIES: &'static [&'static svql_query::session::ExecInfo] = &[
                #(#dep_entries),*
            ];

            fn composite_rehydrate(
                row: &svql_query::session::Row<Self>,
                store: &svql_query::session::Store,
                driver: &svql_query::driver::Driver,
                key: &svql_query::driver::DriverKey,
            ) -> Option<Self> {
                #(#submodule_rehydrate)*
                #(#alias_rehydrate)*  // ADD THIS LINE

                Some(Self {
                    #(#submodule_names,)*
                    #(#alias_names),*  // ADD THIS LINE
                })
            }

            fn preload_driver(
                driver: &svql_query::driver::Driver,
                design_key: &svql_query::driver::DriverKey,
                config: &svql_query::common::Config,
            ) -> Result<(), Box<dyn std::error::Error>>
            where
                Self: Sized,
            {
                #(#preload_calls)*
                Ok(())
            }
        }

        impl #impl_generics svql_query::traits::Component for #name #ty_generics #where_clause {
            type Kind = svql_query::traits::kind::Composite;
        }
    };

    TokenStream::from(expanded)
}

fn parse_connection_attrs(input: &DeriveInput) -> Vec<OrGroup> {
    let mut groups = Vec::new();

    // Parse #[connection(from = [...], to = [...])] - single required connection
    for attr in find_all_attrs(&input.attrs, "connection") {
        if let Some(conn) = parse_single_connection(attr) {
            groups.push(OrGroup {
                connections: vec![conn],
            });
        }
    }

    // Parse #[or_to(from = [...], to = [[...], [...]])] - single source, OR destinations
    for attr in find_all_attrs(&input.attrs, "or_to") {
        if let Some(group) = parse_or_to(attr) {
            groups.push(group);
        }
    }

    // Parse #[or_from(from = [[...], [...]], to = [...])] - OR sources, single destination
    for attr in find_all_attrs(&input.attrs, "or_from") {
        if let Some(group) = parse_or_from(attr) {
            groups.push(group);
        }
    }

    // Parse #[or_group(connection(...), connection(...))] - arbitrary OR group
    for attr in find_all_attrs(&input.attrs, "or_group") {
        if let Some(group) = parse_or_group(attr) {
            groups.push(group);
        }
    }

    groups
}

fn parse_single_connection(attr: &syn::Attribute) -> Option<Connection> {
    let mut from = None;
    let mut to = None;

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("from") {
            let value: ExprArray = meta.value()?.parse()?;
            from = Some(PathSelector::from_expr_array(&value)?);
        } else if meta.path.is_ident("to") {
            let value: ExprArray = meta.value()?.parse()?;
            to = Some(PathSelector::from_expr_array(&value)?);
        } else {
            return Err(meta.error("Expected 'from' or 'to'"));
        }
        Ok(())
    });

    match (from, to) {
        (Some(f), Some(t)) => Some(Connection { from: f, to: t }),
        _ => {
            abort!(attr, "connection attribute requires both 'from' and 'to'");
        }
    }
}

fn parse_or_to(attr: &syn::Attribute) -> Option<OrGroup> {
    let mut from = None;
    let mut to_options: Vec<PathSelector> = Vec::new();

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("from") {
            let value: ExprArray = meta.value()?.parse()?;
            from = Some(PathSelector::from_expr_array(&value)?);
        } else if meta.path.is_ident("to") {
            let value: ExprArray = meta.value()?.parse()?;
            to_options = parse_nested_paths(&value)?;
        } else {
            return Err(meta.error("Expected 'from' or 'to'"));
        }
        Ok(())
    });

    let from = from.unwrap_or_else(|| abort!(attr, "or_to requires 'from'"));
    if to_options.is_empty() {
        abort!(attr, "or_to requires 'to' with at least one destination");
    }

    let connections = to_options
        .into_iter()
        .map(|to| Connection {
            from: from.clone(),
            to,
        })
        .collect();

    Some(OrGroup { connections })
}

fn parse_or_from(attr: &syn::Attribute) -> Option<OrGroup> {
    let mut from_options: Vec<PathSelector> = Vec::new();
    let mut to = None;

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("from") {
            let value: ExprArray = meta.value()?.parse()?;
            from_options = parse_nested_paths(&value)?;
        } else if meta.path.is_ident("to") {
            let value: ExprArray = meta.value()?.parse()?;
            to = Some(PathSelector::from_expr_array(&value)?);
        } else {
            return Err(meta.error("Expected 'from' or 'to'"));
        }
        Ok(())
    });

    if from_options.is_empty() {
        abort!(attr, "or_from requires 'from' with at least one source");
    }
    let to = to.unwrap_or_else(|| abort!(attr, "or_from requires 'to'"));

    let connections = from_options
        .into_iter()
        .map(|from| Connection {
            from,
            to: to.clone(),
        })
        .collect();

    Some(OrGroup { connections })
}

fn parse_or_group(attr: &syn::Attribute) -> Option<OrGroup> {
    let mut connections = Vec::new();

    // Parse nested connection(...) items
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("connection") {
            let content;
            syn::parenthesized!(content in meta.input);

            let mut from = None;
            let mut to = None;

            let nested = content.parse_terminated(Meta::parse, Token![,])?;
            for item in nested {
                if let Meta::NameValue(nv) = item {
                    let key = nv
                        .path
                        .get_ident()
                        .map(std::string::ToString::to_string)
                        .unwrap_or_default();

                    if let Expr::Array(arr) = &nv.value {
                        match key.as_str() {
                            "from" => from = Some(PathSelector::from_expr_array(arr)?),
                            "to" => to = Some(PathSelector::from_expr_array(arr)?),
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    &nv,
                                    "Expected 'from' or 'to'",
                                ));
                            }
                        }
                    }
                }
            }

            if let (Some(f), Some(t)) = (from, to) {
                connections.push(Connection { from: f, to: t });
            }
        }
        Ok(())
    });

    if connections.is_empty() {
        abort!(attr, "or_group requires at least one connection");
    }

    Some(OrGroup { connections })
}

fn parse_submodule_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<SubmoduleField> {
    let mut submodules = Vec::new();

    for field in fields {
        if find_attr(&field.attrs, "submodule").is_some() {
            let name = field
                .ident
                .clone()
                .unwrap_or_else(|| abort!(field, "Submodule fields must be named"));
            submodules.push(SubmoduleField {
                name,
                ty: field.ty.clone(),
            });
        }
    }

    submodules
}

fn parse_alias_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<AliasField> {
    let mut aliases = Vec::new();

    for field in fields {
        let Some(attr) = find_attr(&field.attrs, "alias") else {
            continue;
        };

        let name = field
            .ident
            .clone()
            .unwrap_or_else(|| abort!(field, "Alias fields must be named"));

        let mut direction = None;
        let mut target = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("input") {
                direction = Some(Direction::Input);
            } else if meta.path.is_ident("output") {
                direction = Some(Direction::Output);
            } else if meta.path.is_ident("inout") {
                direction = Some(Direction::Inout);
            } else if meta.path.is_ident("target") {
                let value: ExprArray = meta.value()?.parse()?;
                target = Some(PathSelector::from_expr_array(&value)?);
            } else {
                return Err(meta.error("Unknown alias attribute"));
            }
            Ok(())
        });

        let direction = direction.unwrap_or_else(|| {
            abort!(
                attr,
                "Alias must specify direction: input, output, or inout"
            )
        });
        let target = target.unwrap_or_else(|| abort!(attr, "Alias must specify target path"));

        aliases.push(AliasField {
            name,
            direction,
            target,
        });
    }

    aliases
}
