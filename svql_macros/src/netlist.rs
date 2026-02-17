//! Procedural macro implementation for the `Netlist` derive.
//!
//! Maps struct fields to external Verilog/JSON ports and generates
//! the logic for structural subgraph isomorphism searches.

use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Meta, Token, parse_macro_input};

use crate::parsing::{Direction, find_attr, get_string_value};

/// Parsed netlist attribute data
struct NetlistAttr {
    /// Path to the design source file.
    file: String,
    /// Name of the module within the source file.
    module: String,
}

/// Parsed port field data
struct PortField {
    /// Field identifier.
    name: syn::Ident,
    /// Port direction (Input, Output, etc.).
    direction: Direction,
    /// Optional override for the port name in the source.
    rename: Option<String>,
}

/// Implementation of the `Netlist` procedural macro.
pub fn netlist_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // Ensure it's a struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => abort!(
                input,
                "Netlist derive only supports structs with named fields"
            ),
        },
        _ => abort!(input, "Netlist derive only supports structs"),
    };

    // Parse #[netlist(file = "...", module = "...")]
    let netlist_attr = parse_netlist_attr(&input);

    // Parse port fields
    let ports = parse_port_fields(fields);

    // Generate implementation
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let file_path = &netlist_attr.file;
    let module_name = &netlist_attr.module;

    // Generate PORTS array
    let port_entries: Vec<_> = ports
        .iter()
        .map(|p| {
            let default = p.name.to_string();
            let port_name = p.rename.as_ref().unwrap_or(&default);
            let constructor = p.direction.as_port_constructor();
            quote! { #constructor(#port_name) }
        })
        .collect();

    // Generate rehydrate field assignments
    let field_assignments: Vec<_> = ports
        .iter()
        .map(|p| {
            let field_name = &p.name; // ← Use Ident directly instead of converting to String
            let default = p.name.to_string();
            let port_name = p.rename.as_ref().unwrap_or(&default);
            quote! {
                #field_name: row.wire(#port_name)?
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics svql_query::traits::Netlist for #name #ty_generics #where_clause {
            const MODULE_NAME: &'static str = #module_name;
            const FILE_PATH: &'static str = #file_path;

            const PORTS: &'static [svql_common::PortDecl] = &[
                #(#port_entries),*
            ];

            fn netlist_schema() -> &'static svql_query::session::PatternSchema {
                static SCHEMA: std::sync::OnceLock<svql_query::session::PatternSchema> =
                    std::sync::OnceLock::new();
                SCHEMA.get_or_init(|| {
                    let mut defs = Self::ports_to_defs();

                    // Load needle design to discover internal cells
                    let result = std::panic::catch_unwind(|| {
                        Self::discover_internal_cells()
                    });

                    match result {
                        Ok(Ok(internal_defs)) => {
                            tracing::debug!(
                                "[NETLIST] {} discovered {} internal cells",
                                std::any::type_name::<Self>(),
                                internal_defs.len()
                            );
                            defs.extend(internal_defs);
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(
                                "[NETLIST] {} failed to load needle during schema init: {}",
                                std::any::type_name::<Self>(),
                                e
                            );
                        }
                        Err(panic_val) => {
                            tracing::error!(
                                "[NETLIST] {} panicked during needle loading: {:?}",
                                std::any::type_name::<Self>(),
                                panic_val
                            );
                        }
                    }

                    let defs_static: &'static [svql_query::session::ColumnDef] =
                        Box::leak(defs.into_boxed_slice());
                    svql_query::session::PatternSchema::new(defs_static)
                })
            }

            fn netlist_rehydrate(
                row: &svql_query::session::Row<Self>,
                _store: &svql_query::session::Store,
                _driver: &svql_query::driver::Driver,
                _key: &svql_query::driver::DriverKey,
            ) -> Option<Self>
            where
                Self: svql_query::traits::Component
                    + svql_query::traits::PatternInternal<svql_query::traits::kind::Netlist>
                    + Send + Sync + 'static,
            {
                Some(Self {
                    #(#field_assignments),*
                })
            }
        }

        impl #impl_generics svql_query::traits::Component for #name #ty_generics #where_clause {
            type Kind = svql_query::traits::kind::Netlist;
        }
    };

    TokenStream::from(expanded)
}

/// Extracts the `#[netlist(...)]` container attribute.
fn parse_netlist_attr(input: &DeriveInput) -> NetlistAttr {
    let attr = find_attr(&input.attrs, "netlist").unwrap_or_else(|| {
        abort!(
            input,
            "Missing #[netlist(file = \"...\", module = \"...\")] attribute"
        )
    });

    let mut file = None;
    let mut module = None;

    let meta_list = attr
        .parse_args_with(|input: syn::parse::ParseStream| {
            syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated(input)
        })
        .unwrap_or_else(|e| abort!(attr, "Failed to parse netlist attribute: {}", e));

    for meta in meta_list {
        if let Meta::NameValue(nv) = meta {
            let key = nv
                .path
                .get_ident()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();

            match key.as_str() {
                "file" => {
                    file = Some(get_string_value(&nv).unwrap_or_else(|e| abort!(nv, "{}", e)));
                }
                "module" => {
                    module = Some(get_string_value(&nv).unwrap_or_else(|e| abort!(nv, "{}", e)));
                }
                other => abort!(nv, "Unknown netlist attribute key: {}", other),
            }
        }
    }

    NetlistAttr {
        file: file.unwrap_or_else(|| abort!(attr, "Missing 'file' in netlist attribute")),
        module: module.unwrap_or_else(|| abort!(attr, "Missing 'module' in netlist attribute")),
    }
}

/// Identifies fields marked with `#[port(...)]` and extracts their connectivity requirements.
fn parse_port_fields(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> Vec<PortField> {
    let mut ports = Vec::new();

    for field in fields {
        let Some(port_attr) = find_attr(&field.attrs, "port") else {
            continue;
        };

        let field_name = field
            .ident
            .clone()
            .unwrap_or_else(|| abort!(field, "Port fields must be named"));

        // Parse #[port(input)] or #[port(output, rename = "...")]
        let (direction, rename) = parse_port_attr(port_attr);

        ports.push(PortField {
            name: field_name,
            direction,
            rename,
        });
    }

    if ports.is_empty() {
        abort!(
            fields,
            "At least one field must have a #[port(...)] attribute"
        );
    }

    ports
}

/// Parses a single `#[port(...)]` field attribute.
fn parse_port_attr(attr: &syn::Attribute) -> (Direction, Option<String>) {
    let mut direction = None;
    let mut rename = None;

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("input") {
            direction = Some(Direction::Input);
        } else if meta.path.is_ident("output") {
            direction = Some(Direction::Output);
        } else if meta.path.is_ident("inout") {
            direction = Some(Direction::Inout);
        } else if meta.path.is_ident("rename") {
            let value: syn::LitStr = meta.value()?.parse()?;
            rename = Some(value.value());
        } else {
            return Err(meta.error("Unknown port attribute"));
        }
        Ok(())
    });

    let dir = direction
        .unwrap_or_else(|| abort!(attr, "Port must specify direction: input, output, or inout"));

    (dir, rename)
}
