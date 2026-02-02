// svql_macros/src/variant.rs
#![allow(clippy::too_many_lines, clippy::cast_possible_truncation)]

use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DeriveInput, ExprArray, Fields, Ident, parse_macro_input};

use crate::parsing::{Direction, PathSelector, find_attr};

/// A common port declaration
struct CommonPort {
    name: String,
    direction: Direction,
}

/// A port mapping from common port to inner path
struct PortMapping {
    common_port: String,
    inner_path: PathSelector,
}

/// A variant arm with its type and port mappings
struct VariantArm {
    name: Ident,
    inner_ty: syn::Type,
    mappings: Vec<PortMapping>,
}

pub fn variant_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // Ensure it's an enum
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => abort!(input, "Variant derive only supports enums"),
    };

    // Parse #[variant_ports(input(a), output(y))]
    let common_ports = parse_variant_ports(&input);

    // Parse each variant arm
    let arms = parse_variant_arms(variants);

    // Generate implementation
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let num_variants = arms.len();

    // Generate COMMON_PORTS array
    let port_entries: Vec<_> = common_ports
        .iter()
        .map(|p| {
            let port_name = &p.name;
            let constructor = p.direction.as_port_constructor();
            quote! { #constructor(#port_name) }
        })
        .collect();

    // Generate PORT_MAPPINGS array (one inner array per variant)
    let mapping_arrays: Vec<_> = arms
        .iter()
        .map(|arm| {
            let mappings: Vec<_> = arm
                .mappings
                .iter()
                .map(|m| {
                    let common = &m.common_port;
                    let inner = m.inner_path.to_selector_tokens();
                    quote! {
                        svql_query::session::PortMap::new(#common, #inner)
                    }
                })
                .collect();
            quote! {
                &[#(#mappings),*]
            }
        })
        .collect();

    // Generate VARIANT_ARMS array
    let arm_entries: Vec<_> = arms
        .iter()
        .map(|arm| {
            let ty = &arm.inner_ty;
            let type_name = arm.name.to_string();
            quote! {
                svql_query::traits::variant::VariantArm {
                    type_id: std::any::TypeId::of::<#ty>(),
                    type_name: #type_name,
                }
            }
        })
        .collect();

    // Generate DEPENDANCIES array
    let dep_entries: Vec<_> = arms
        .iter()
        .map(|arm| {
            let ty = &arm.inner_ty;
            quote! {
                <#ty as svql_query::traits::Pattern>::EXEC_INFO
            }
        })
        .collect();

    // Generate rehydrate match arms
    let rehydrate_arms: Vec<_> = arms
        .iter()
        .enumerate()
        .map(|(idx, arm)| {
            let variant_name = &arm.name;
            let ty = &arm.inner_ty;
            let idx_lit = idx as u32;
            quote! {
                #idx_lit => {
                    let inner_table = store.get::<#ty>()?;
                    let inner_row = inner_table.row(inner_row_idx)?;
                    let inner = #ty::rehydrate(
                        &inner_row, store, driver, key
                    )?;
                    Some(Self::#variant_name(inner))
                }
            }
        })
        .collect();

    // Generate preload_driver calls
    let preload_calls: Vec<_> = arms
        .iter()
        .map(|arm| {
            let ty = &arm.inner_ty;
            quote! {
                <#ty as svql_query::traits::Pattern>::preload_driver(driver, design_key, config)?;
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics svql_query::traits::variant::Variant for #name #ty_generics #where_clause {
            const NUM_VARIANTS: usize = #num_variants;

            const COMMON_PORTS: &'static [svql_query::session::Port] = &[
                #(#port_entries),*
            ];

            const PORT_MAPPINGS: &'static [&'static [svql_query::session::PortMap]] = &[
                #(#mapping_arrays),*
            ];

            const VARIANT_ARMS: &'static [svql_query::traits::variant::VariantArm] = &[
                #(#arm_entries),*
            ];

            const DEPENDANCIES: &'static [&'static svql_query::session::ExecInfo] = &[
                #(#dep_entries),*
            ];

            fn variant_rehydrate(
                row: &svql_query::session::Row<Self>,
                store: &svql_query::session::Store,
                driver: &svql_query::driver::Driver,
                key: &svql_query::driver::DriverKey,
            ) -> Option<Self>
            where
                Self: svql_query::traits::Component
                    + svql_query::traits::PatternInternal<svql_query::traits::kind::Variant>
                    + Send + Sync + 'static,
            {
                // Get discriminant
                let discrim_idx = Self::schema()
                    .index_of("discriminant")?;
                let discrim = row.entry_array().entries.get(discrim_idx)?.as_u32()?;

                // Get inner_ref
                let inner_ref_idx = Self::schema()
                    .index_of("inner_ref")?;
                let inner_row_idx = row.entry_array().entries.get(inner_ref_idx)?.as_u32()?;

                // Dispatch based on discriminant
                match discrim {
                    #(#rehydrate_arms)*
                    _ => None,
                }
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
            type Kind = svql_query::traits::kind::Variant;
        }
    };

    TokenStream::from(expanded)
}

fn parse_variant_ports(input: &DeriveInput) -> Vec<CommonPort> {
    let attr = find_attr(&input.attrs, "variant_ports")
        .unwrap_or_else(|| abort!(input, "Missing #[variant_ports(...)] attribute"));

    let mut ports = Vec::new();

    let _ = attr.parse_nested_meta(|meta| {
        // Parse input(name) or output(name)
        let direction = if meta.path.is_ident("input") {
            Direction::Input
        } else if meta.path.is_ident("output") {
            Direction::Output
        } else if meta.path.is_ident("inout") {
            Direction::Inout
        } else {
            return Err(meta.error("Expected 'input', 'output', or 'inout'"));
        };

        let content;
        syn::parenthesized!(content in meta.input);
        let port_name: Ident = content.parse()?;

        ports.push(CommonPort {
            name: port_name.to_string(),
            direction,
        });

        Ok(())
    });

    if ports.is_empty() {
        abort!(attr, "variant_ports requires at least one port");
    }

    ports
}

fn parse_variant_arms(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Vec<VariantArm> {
    let mut arms = Vec::new();

    for variant in variants {
        // Get the inner type (must be a single tuple field)
        let inner_ty = match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                fields.unnamed.first().unwrap().ty.clone()
            }
            _ => abort!(
                variant,
                "Variant arms must have exactly one tuple field: VariantName(InnerType)"
            ),
        };

        // Parse #[map(a = ["..."], b = ["..."])]
        let map_attr = find_attr(&variant.attrs, "map")
            .unwrap_or_else(|| abort!(variant, "Missing #[map(...)] attribute on variant arm"));

        let mappings = parse_port_mappings(map_attr);

        arms.push(VariantArm {
            name: variant.ident.clone(),
            inner_ty,
            mappings,
        });
    }

    arms
}

fn parse_port_mappings(attr: &syn::Attribute) -> Vec<PortMapping> {
    let mut mappings = Vec::new();

    let _ = attr.parse_nested_meta(|meta| {
        let port_name = meta
            .path
            .get_ident()
            .map(std::string::ToString::to_string)
            .ok_or_else(|| meta.error("Expected port name"))?;

        let value: ExprArray = meta.value()?.parse()?;
        let inner_path = PathSelector::from_expr_array(&value)?;

        mappings.push(PortMapping {
            common_port: port_name,
            inner_path,
        });

        Ok(())
    });

    mappings
}
