//! Procedural macro implementation for the `Composite` derive.

#![allow(clippy::too_many_lines, clippy::unnecessary_wraps)]

use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprArray, Fields, Meta, Token, parse::Parse, parse_macro_input,
};

use crate::parsing::{Direction, PathSelector, find_all_attrs, find_attr, parse_nested_paths};

/// Represents a connection between two paths in a composite pattern.
struct Connection {
    /// The source path of the connection.
    from: PathSelector,
    /// The destination path of the connection.
    to: PathSelector,
    /// Optional connection kind modifier.
    kind: Option<ConnectionKind>,
}

/// Specifies the matching behavior for a connection.
enum ConnectionKind {
    /// Match if any element in a set satisfies the connection.
    AnyInSet,
}

/// A group of connections where at least one must be satisfied.
struct OrGroup {
    /// The connections in this OR group.
    connections: Vec<Connection>,
}

/// A custom filter expression for validation.
struct Filter {
    /// The filter expression to evaluate.
    expr: syn::Expr,
}

/// Represents a submodule field in a composite pattern.
struct SubmoduleField {
    /// The field name.
    name: syn::Ident,
    /// The field type.
    ty: syn::Type,
}

/// Represents an alias field in a composite pattern.
struct AliasField {
    /// The alias field name.
    name: syn::Ident,
    /// The port direction (input/output).
    direction: Direction,
    /// The target path this alias points to.
    target: PathSelector,
}

/// Implementation of the `Composite` derive macro.
pub fn composite_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => abort!(input, "Composite derive only supports structs with named fields"),
        },
        _ => abort!(input, "Composite derive only supports structs"),
    };

    let or_groups = parse_or_groups(&input);
    let filters = parse_filters(&input);
    let submodules = parse_submodule_fields(fields);
    let aliases = parse_alias_fields(fields);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let submodule_entries: Vec<_> = submodules
        .iter()
        .map(|s| {
            let field_name = s.name.to_string();
            let ty = &s.ty;
            quote! { svql_query::session::Submodule::of::<#ty>(#field_name) }
        })
        .collect();

    let alias_entries: Vec<_> = aliases
        .iter()
        .map(|a| {
            let port_name = a.name.to_string();
            let target = a.target.to_selector_tokens();
            let constructor = match a.direction {
                Direction::Output => quote! { svql_query::session::Alias::output },
                Direction::Input | Direction::Inout => quote! { svql_query::session::Alias::input },
            };
            quote! { #constructor(#port_name, #target) }
        })
        .collect();

    let connection_groups: Vec<_> = or_groups
        .iter()
        .map(|group| {
            let connections: Vec<_> = group
                .connections
                .iter()
                .map(|conn| {
                    let from = conn.from.to_selector_tokens();
                    let to = conn.to.to_selector_tokens();
                    match &conn.kind {
                        Some(ConnectionKind::AnyInSet) => quote! {
                            svql_query::traits::composite::Connection::any_in_set(#from, #to)
                        },
                        None => quote! {
                            svql_query::traits::composite::Connection::new(#from, #to)
                        },
                    }
                })
                .collect();
            quote! { &[#(#connections),*] }
        })
        .collect();

    let dep_entries: Vec<_> = submodules
        .iter()
        .map(|s| {
            let ty = &s.ty;
            quote! { <#ty as svql_query::traits::Pattern>::EXEC_INFO }
        })
        .collect();

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
                    let sub_row = sub_table.row(sub_ref)?;
                    #ty::rehydrate(&sub_row, store, driver, key, config)?
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
                let #field_name = row.wire(#field_str)?.clone();
            }
        })
        .collect();

    let submodule_names: Vec<_> = submodules.iter().map(|s| &s.name).collect();
    let alias_names: Vec<_> = aliases.iter().map(|a| &a.name).collect();

    let preload_calls: Vec<_> = submodules
        .iter()
        .map(|s| {
            let ty = &s.ty;
            quote! {
                <#ty as svql_query::traits::Pattern>::preload_driver(driver, design_key, config)?;
            }
        })
        .collect();

    let validate_custom_impl = if !filters.is_empty() {
        generate_validate_custom(&filters)
    } else {
        quote! {}
    };

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

            fn composite_schema() -> &'static svql_query::session::PatternSchema {
                static SCHEMA: std::sync::OnceLock<svql_query::session::PatternSchema> =
                    std::sync::OnceLock::new();
                SCHEMA.get_or_init(|| {
                    let defs = Self::composite_to_defs();
                    let defs_static: &'static [svql_query::session::ColumnDef] =
                        Box::leak(defs.into_boxed_slice());
                    svql_query::session::PatternSchema::new(defs_static)
                })
            }

            #validate_custom_impl

            fn composite_rehydrate(
                row: &svql_query::session::Row<Self>,
                store: &svql_query::session::Store,
                driver: &svql_query::driver::Driver,
                key: &svql_query::driver::DriverKey,
                config: &svql_query::common::Config,
            ) -> Option<Self> {
                #(#submodule_rehydrate)*
                #(#alias_rehydrate)*

                Some(Self {
                    #(#submodule_names,)*
                    #(#alias_names),*
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

/// Parses OR groups from derive input attributes.
fn parse_or_groups(input: &DeriveInput) -> Vec<OrGroup> {
    let mut groups = Vec::new();

    for attr in find_all_attrs(&input.attrs, "connection") {
        if let Some(conn) = parse_single_connection(attr) {
            groups.push(OrGroup { connections: vec![conn] });
        }
    }

    for attr in find_all_attrs(&input.attrs, "or_to") {
        if let Some(group) = parse_or_to(attr) {
            groups.push(group);
        }
    }

    for attr in find_all_attrs(&input.attrs, "or_from") {
        if let Some(group) = parse_or_from(attr) {
            groups.push(group);
        }
    }

    for attr in find_all_attrs(&input.attrs, "or_group") {
        if let Some(group) = parse_or_group(attr) {
            groups.push(group);
        }
    }

    groups
}

/// Parses a single connection attribute.
fn parse_single_connection(attr: &syn::Attribute) -> Option<Connection> {
    let mut from = None;
    let mut to = None;
    let mut kind = None;

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("from") {
            let value: ExprArray = meta.value()?.parse()?;
            from = Some(PathSelector::from_expr_array(&value)?);
        } else if meta.path.is_ident("to") {
            let value: ExprArray = meta.value()?.parse()?;
            to = Some(PathSelector::from_expr_array(&value)?);
        } else if meta.path.is_ident("kind") {
            let value: syn::Lit = meta.value()?.parse()?;
            if let syn::Lit::Str(lit_str) = value {
                let kind_str = lit_str.value();
                if kind_str == "any" {
                    kind = Some(ConnectionKind::AnyInSet);
                } else {
                    abort!(attr, "connection kind must be 'any', got '{}'", kind_str);
                }
            }
        } else {
            return Err(meta.error("Expected 'from', 'to', or 'kind'"));
        }
        Ok(())
    });

    match (from, to) {
        (Some(f), Some(t)) => Some(Connection { from: f, to: t, kind }),
        _ => abort!(attr, "connection attribute requires both 'from' and 'to'"),
    }
}

/// Parses an `or_to` attribute with one source and multiple destinations.
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
        .map(|to| Connection { from: from.clone(), to, kind: None })
        .collect();

    Some(OrGroup { connections })
}

/// Parses an `or_from` attribute with multiple sources and one destination.
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
        .map(|from| Connection { from, to: to.clone(), kind: None })
        .collect();

    Some(OrGroup { connections })
}

/// Parses an `or_group` attribute containing multiple connections.
fn parse_or_group(attr: &syn::Attribute) -> Option<OrGroup> {
    let mut connections = Vec::new();

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
                            _ => return Err(syn::Error::new_spanned(&nv, "Expected 'from' or 'to'")),
                        }
                    }
                }
            }

            if let (Some(f), Some(t)) = (from, to) {
                connections.push(Connection { from: f, to: t, kind: None });
            }
        }
        Ok(())
    });

    if connections.is_empty() {
        abort!(attr, "or_group requires at least one connection");
    }

    Some(OrGroup { connections })
}

/// Parses all filter attributes from the derive input.
fn parse_filters(input: &DeriveInput) -> Vec<Filter> {
    find_all_attrs(&input.attrs, "filter")
        .into_iter()
        .filter_map(parse_single_filter)
        .collect()
}

/// Parses a single filter attribute.
fn parse_single_filter(attr: &syn::Attribute) -> Option<Filter> {
    let expr = match attr.parse_args::<syn::Expr>() {
        Ok(expr) => expr,
        Err(e) => abort!(attr, "filter attribute expects a function path or closure: {}", e),
    };

    match &expr {
        syn::Expr::Path(_) | syn::Expr::Closure(_) => {}
        _ => abort!(
            attr,
            "filter must be a function path or closure (e.g., |row, ctx| ...)"
        ),
    }

    Some(Filter { expr })
}

/// Extracts submodule fields from the struct fields.
fn parse_submodule_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<SubmoduleField> {
    fields
        .iter()
        .filter(|f| find_attr(&f.attrs, "submodule").is_some())
        .map(|f| SubmoduleField {
            name: f.ident.clone().unwrap_or_else(|| abort!(f, "Submodule fields must be named")),
            ty: f.ty.clone(),
        })
        .collect()
}

/// Extracts alias fields from the struct fields.
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
            abort!(attr, "Alias must specify direction: input, output, or inout")
        });
        let target = target.unwrap_or_else(|| abort!(attr, "Alias must specify target path"));

        aliases.push(AliasField { name, direction, target });
    }

    aliases
}

/// Generates the code for custom filter validation.
fn generate_validate_custom(filters: &[Filter]) -> proc_macro2::TokenStream {
    let filter_calls: Vec<_> = filters
        .iter()
        .map(|f| {
            let expr = &f.expr;
            quote! {
                if !(#expr)(row, ctx) {
                    return false;
                }
            }
        })
        .collect();

    quote! {
        fn validate_custom(
            row: &svql_query::session::Row<Self>,
            ctx: &svql_query::session::ExecutionContext,
        ) -> bool {
            #(#filter_calls)*
            true
        }
    }
}
