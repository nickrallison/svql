//! Procedural macro implementation for the `Variant` derive.

use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::{Data, DeriveInput, ExprArray, Fields, Ident, parse_macro_input};

use crate::parsing::{Direction, PathSelector, find_attr};

/// Represents a common port shared across all variants.
struct CommonPort {
    /// The port name.
    name: String,
    /// The port direction (input/output).
    direction: Direction,
}

/// Maps a common port to a path in a variant's inner type.
struct PortMapping {
    /// The name of the common port.
    common_port: String,
    /// The path to the inner port in the variant.
    inner_path: PathSelector,
}

/// Represents a variant arm in the enum.
struct VariantArm {
    /// The variant name.
    name: Ident,
    /// The inner type wrapped by this variant.
    inner_ty: syn::Type,
    /// Port mappings for this variant.
    mappings: Vec<PortMapping>,
}

/// Implementation of the `Variant` derive macro.
pub fn variant_impl(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => abort!(input, "Variant derive only supports enums"),
    };

    let common_ports = parse_variant_ports(&input);
    let arms = parse_variant_arms(variants);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let num_variants = arms.len();

    let port_entries: Vec<_> = common_ports
        .iter()
        .map(|p| {
            let port_name = &p.name;
            let constructor = p.direction.as_port_constructor();
            quote! { #constructor(#port_name) }
        })
        .collect();

    let mapping_arrays: Vec<_> = arms
        .iter()
        .map(|arm| {
            let mappings: Vec<_> = arm
                .mappings
                .iter()
                .map(|m| {
                    let common = &m.common_port;
                    let inner = m.inner_path.to_selector_tokens();
                    quote! { svql_query::session::PortMap::new(#common, #inner) }
                })
                .collect();
            quote! { &[#(#mappings),*] }
        })
        .collect();

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

    let dep_entries: Vec<_> = arms
        .iter()
        .map(|arm| {
            let ty = &arm.inner_ty;
            quote! { <#ty as svql_query::traits::Pattern>::EXEC_INFO }
        })
        .collect();

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
                    let inner_row = inner_table.row_at(inner_row_idx)?;
                    let inner = #ty::rehydrate(&inner_row, store, driver, key, config)?;;
                    Some(Self::#variant_name(inner))
                }
            }
        })
        .collect();

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

            const COMMON_PORTS: &'static [svql_query::common::PortDecl] = &[
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

            fn variant_schema() -> &'static svql_query::session::PatternSchema {
                static SCHEMA: std::sync::OnceLock<svql_query::session::PatternSchema> =
                    std::sync::OnceLock::new();
                SCHEMA.get_or_init(|| {
                    let defs = Self::variant_to_defs();
                    let defs_static: &'static [svql_query::session::ColumnDef] =
                        Box::leak(defs.into_boxed_slice());
                    svql_query::session::PatternSchema::new(defs_static)
                })
            }

            fn variant_rehydrate(
                row: &svql_query::session::Row<Self>,
                store: &svql_query::session::Store,
                driver: &svql_query::driver::Driver,
                key: &svql_query::driver::DriverKey,
                config: &svql_query::common::Config,
            ) -> Option<Self>
            where
                Self: svql_query::traits::Component
                    + svql_query::traits::PatternInternal<svql_query::traits::kind::Variant>
                    + 'static,
            {
                let discrim = row.meta("discriminant")?.as_discriminant()?.raw();
                let inner_row_idx = row.meta("inner_ref")?.as_count()?;

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

/// Parses the common ports from the `#[variant_ports(...)]` attribute.
fn parse_variant_ports(input: &DeriveInput) -> Vec<CommonPort> {
    let attr = find_attr(&input.attrs, "variant_ports")
        .unwrap_or_else(|| abort!(input, "Missing #[variant_ports(...)] attribute"));

    let mut ports = Vec::new();

    let _ = attr.parse_nested_meta(|meta| {
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

/// Parses all variant arms and their port mappings.
fn parse_variant_arms(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Vec<VariantArm> {
    variants
        .iter()
        .map(|variant| {
            let inner_ty = match &variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    fields.unnamed.first().unwrap().ty.clone()
                }
                _ => abort!(
                    variant,
                    "Variant arms must have exactly one tuple field: VariantName(InnerType)"
                ),
            };

            let map_attr = find_attr(&variant.attrs, "map")
                .unwrap_or_else(|| abort!(variant, "Missing #[map(...)] attribute on variant arm"));

            VariantArm {
                name: variant.ident.clone(),
                inner_ty,
                mappings: parse_port_mappings(map_attr),
            }
        })
        .collect()
}

/// Parses port mappings from a `#[map(...)]` attribute.
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
