use crate::common;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, parse_macro_input};

enum FieldKind {
    Submodule,
    Wire,
}

struct CompositeField {
    ident: syn::Ident,
    ty: syn::Type,
    vis: syn::Visibility,
    kind: FieldKind,
}

pub fn composite_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &item_struct.ident;
    let struct_name_str = struct_name.to_string();

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();
    let specialized_generics = common::remove_state_generic(&item_struct.generics);
    let (spec_impl_generics, _, spec_where_clause) = specialized_generics.split_for_impl();

    let search_type = common::make_replaced_type(
        struct_name,
        &item_struct.generics,
        quote!(::svql_query::Search),
    );
    let match_type = common::make_replaced_type(
        struct_name,
        &item_struct.generics,
        quote!(::svql_query::Match),
    );

    let mut fields_info = Vec::new();

    if let Fields::Named(ref fields) = item_struct.fields {
        for field in &fields.named {
            let ident = field.ident.clone().unwrap();

            // Skip "path" if the user manually added it
            if ident == "path" {
                continue;
            }

            let mut kind = FieldKind::Wire;
            for attr in &field.attrs {
                if attr.path().is_ident("submodule") {
                    kind = FieldKind::Submodule;
                }
            }

            fields_info.push(CompositeField {
                ident,
                ty: field.ty.clone(),
                vis: field.vis.clone(),
                kind,
            });
        }
    }

    // --- Generation Phase ---

    let struct_fields = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let vis = &f.vis;
        quote! { #vis #ident: #ty }
    });

    let instantiate_fields = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let name_str = ident.to_string();
        match f.kind {
            FieldKind::Submodule => {
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                quote! {
                   #ident: <#search_ty as ::svql_query::traits::SearchableComponent>::create_at(base_path.child(#name_str))
                }
            },
            FieldKind::Wire => quote! {
                #ident: ::svql_query::Wire::new(base_path.child(#name_str), ())
            },
        }
    });

    let mut query_calls = Vec::new();
    let mut query_vars = Vec::new();
    let mut construct_fields = Vec::new();

    for f in &fields_info {
        let ident = &f.ident;
        match f.kind {
            FieldKind::Submodule => {
                query_calls.push(quote! {
                    let #ident = self.#ident.execute_search(driver, context, key, config);

                });
                query_vars.push(ident);
                construct_fields.push(quote! { #ident: #ident });
            }
            FieldKind::Wire => {
                construct_fields.push(quote! {
                    #ident: ::svql_query::Wire::new(self.#ident.path.clone(), None)
                });
            }
        }
    }

    let children_impl = fields_info.iter().map(|f| {
        let ident = &f.ident;
        quote! { &self.#ident }
    });

    let context_calls = fields_info.iter().filter_map(|f| {
        if let FieldKind::Submodule = f.kind {
            let ty = &f.ty;
            let search_ty = common::replace_state_generic(ty);
            Some(quote! {
                let sub_ctx = <#search_ty as ::svql_query::traits::SearchableComponent>::build_context(driver, options)?;
                ctx = ctx.merge(sub_ctx);
            })
        } else {
            None
        }
    });

    // --- Dehydrate/Rehydrate Generation ---

    let wire_field_descs: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Wire = f.kind {
                let name = f.ident.to_string();
                Some(quote! {
                    ::svql_query::session::WireFieldDesc { name: #name }
                })
            } else {
                None
            }
        })
        .collect();

    // For submodule field descriptors, extract the base type name from the type path
    let submodule_field_descs: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let name = f.ident.to_string();
                let ty = &f.ty;
                // Extract the type name from the path (e.g., "Sdffe" from "Sdffe<S>")
                let type_name = if let syn::Type::Path(type_path) = ty {
                    type_path
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                } else {
                    "Unknown".to_string()
                };
                Some(quote! {
                    ::svql_query::session::SubmoduleFieldDesc {
                        name: #name,
                        type_name: #type_name,
                    }
                })
            } else {
                None
            }
        })
        .collect();

    // Generate wire-only dehydration (submodules need special handling)
    let dehydrate_wire_only: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Wire = f.kind {
                let ident = &f.ident;
                let name = f.ident.to_string();
                Some(quote! {
                    .with_wire(#name, self.#ident.inner.as_ref().map(|c| c.id as u32))
                })
            } else {
                None
            }
        })
        .collect();

    // Rehydrate fields
    let rehydrate_fields: Vec<_> = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let name = f.ident.to_string();
        match f.kind {
            FieldKind::Wire => {
                quote! {
                    #ident: ctx.rehydrate_wire(
                        ::svql_query::instance::Instance::from_path(&row.path).child(#name),
                        row.wire(#name)
                    )
                }
            }
            FieldKind::Submodule => {
                let ty = &f.ty;
                let match_ty = common::replace_state_generic_with(ty, quote!(::svql_query::Match));
                quote! {
                    #ident: {
                        let sub_idx = row.submodule(#name)
                            .ok_or_else(|| ::svql_query::session::SessionError::RehydrationError(
                                format!("Missing submodule index for {}", #name)
                            ))?;
                        <#match_ty as ::svql_query::session::Rehydrate>::rehydrate_by_index(sub_idx, ctx)?
                    }
                }
            }
        }
    }).collect();

    // Collect submodule types for where clause bounds
    let submodule_dehydrate_bounds: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ty = &f.ty;
                let match_ty = common::replace_state_generic_with(ty, quote!(::svql_query::Match));
                Some(quote! { #match_ty: ::svql_query::session::Dehydrate })
            } else {
                None
            }
        })
        .collect();

    let submodule_rehydrate_bounds: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ty = &f.ty;
                let match_ty = common::replace_state_generic_with(ty, quote!(::svql_query::Match));
                Some(quote! { #match_ty: ::svql_query::session::Rehydrate })
            } else {
                None
            }
        })
        .collect();

    // Check if there are any submodules requiring bounds
    let has_submodules = fields_info
        .iter()
        .any(|f| matches!(f.kind, FieldKind::Submodule));

    let expanded = quote! {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct #struct_name #impl_generics #where_clause {
            pub path: ::svql_query::instance::Instance,
            #(#struct_fields),*
        }

        // Hardware implementation (state-generic)
        impl #impl_generics ::svql_query::prelude::Hardware for #struct_name #ty_generics #where_clause {
            type State = S;

            fn path(&self) -> &::svql_query::prelude::Instance {
                &self.path
            }

            fn type_name(&self) -> &'static str {
                stringify!(#struct_name)
            }

            fn children(&self) -> Vec<&dyn ::svql_query::prelude::Hardware<State = Self::State>> {
                vec![ #(#children_impl),* ]
            }
        }

        // SearchableComponent implementation (Search state)
        impl #spec_impl_generics ::svql_query::traits::SearchableComponent for #search_type #spec_where_clause {
            type Kind = ::svql_query::traits::kind::Composite;
            type Match = #match_type;

            fn create_at(base_path: ::svql_query::prelude::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    #(#instantiate_fields),*
                }
            }

            fn build_context(
                driver: &::svql_query::prelude::Driver,
                options: &::svql_query::prelude::ModuleConfig
            ) -> Result<::svql_query::prelude::Context, Box<dyn std::error::Error>> {
                let mut ctx = ::svql_query::prelude::Context::new();
                #(#context_calls)*
                Ok(ctx)
            }

            fn execute_search(
                &self,
                driver: &::svql_query::prelude::Driver,
                context: &::svql_query::prelude::Context,
                key: &::svql_query::prelude::DriverKey,
                config: &::svql_query::prelude::Config
            ) -> Vec<Self::Match> {
                use ::svql_query::traits::CompositeComponent;
                self.execute_submodules(driver, context, key, config)
            }
        }

        // CompositeComponent implementation (Search state)
        impl #spec_impl_generics ::svql_query::traits::CompositeComponent for #search_type #spec_where_clause {
            fn execute_submodules(
                &self,
                driver: &::svql_query::prelude::Driver,
                context: &::svql_query::prelude::Context,
                key: &::svql_query::prelude::DriverKey,
                config: &::svql_query::prelude::Config
            ) -> Vec<Self::Match> {
                use ::svql_query::prelude::validate_composite;
                use ::svql_query::traits::SearchableComponent;

                // 1. Execute sub-queries
                #(#query_calls)*

                // 2. Cartesian Product & Filtering
                let haystack_index = context.get(key).unwrap().index();
                let mut cache = std::collections::HashMap::new();
                ::svql_query::itertools::iproduct!( #(#query_vars),* )
                    .map(|( #(#query_vars),* )| {
                        #struct_name {
                            path: self.path.clone(),
                            #(#construct_fields),*
                        }
                    })
                    .filter(|candidate| {
                        if let Some(cached) = cache.get(candidate) {
                            *cached
                        } else {
                            let is_valid = validate_composite(candidate, haystack_index);
                            cache.insert(candidate.clone(), is_valid);
                            is_valid
                        }
                    })
                    .collect()
            }
        }

        // MatchedComponent implementation (Match state)
        impl #spec_impl_generics ::svql_query::traits::MatchedComponent for #match_type #spec_where_clause {
            type Search = #search_type;
        }

        // CompositeMatched implementation (Match state)
        impl #spec_impl_generics ::svql_query::traits::CompositeMatched for #match_type #spec_where_clause {
            type SearchType = #search_type;
        }
    };

    // Generate Dehydrate/Rehydrate impls with appropriate where clauses
    let dehydrate_impl = if has_submodules {
        // Need where clause for submodule bounds
        quote! {
            impl #spec_impl_generics ::svql_query::session::Dehydrate for #match_type
            where
                #(#submodule_dehydrate_bounds),*
            {
                const SCHEMA: ::svql_query::session::QuerySchema = ::svql_query::session::QuerySchema::new(
                    #struct_name_str,
                    &[ #(#wire_field_descs),* ],
                    &[ #(#submodule_field_descs),* ],
                );

                fn dehydrate(&self) -> ::svql_query::session::DehydratedRow {
                    // Note: For composites with submodules, the caller must set submodule indices
                    // after calling dehydrate() since those are foreign keys requiring index lookup
                    ::svql_query::session::DehydratedRow::new(self.path.to_string())
                        #(#dehydrate_wire_only)*
                }
            }
        }
    } else {
        // No submodules, no extra bounds needed
        quote! {
            impl #spec_impl_generics ::svql_query::session::Dehydrate for #match_type #spec_where_clause {
                const SCHEMA: ::svql_query::session::QuerySchema = ::svql_query::session::QuerySchema::new(
                    #struct_name_str,
                    &[ #(#wire_field_descs),* ],
                    &[],
                );

                fn dehydrate(&self) -> ::svql_query::session::DehydratedRow {
                    ::svql_query::session::DehydratedRow::new(self.path.to_string())
                        #(#dehydrate_wire_only)*
                }
            }
        }
    };

    let rehydrate_impl = if has_submodules {
        // Need where clause for submodule bounds
        quote! {
            impl #spec_impl_generics ::svql_query::session::Rehydrate for #match_type
            where
                #(#submodule_rehydrate_bounds),*
            {
                const TYPE_NAME: &'static str = #struct_name_str;

                fn rehydrate(
                    row: &::svql_query::session::MatchRow,
                    ctx: &::svql_query::session::RehydrateContext<'_>,
                ) -> Result<Self, ::svql_query::session::SessionError> {
                    Ok(#struct_name {
                        path: ::svql_query::instance::Instance::from_path(&row.path),
                        #(#rehydrate_fields),*
                    })
                }
            }
        }
    } else {
        // No submodules, simpler rehydration
        quote! {
            impl #spec_impl_generics ::svql_query::session::Rehydrate for #match_type #spec_where_clause {
                const TYPE_NAME: &'static str = #struct_name_str;

                fn rehydrate(
                    row: &::svql_query::session::MatchRow,
                    ctx: &::svql_query::session::RehydrateContext<'_>,
                ) -> Result<Self, ::svql_query::session::SessionError> {
                    Ok(#struct_name {
                        path: ::svql_query::instance::Instance::from_path(&row.path),
                        #(#rehydrate_fields),*
                    })
                }
            }
        }
    };

    // --- SearchDehydrate Generation ---

    // Collect SearchDehydrate bounds for submodules
    let submodule_search_dehydrate_bounds: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! { #search_ty: ::svql_query::session::SearchDehydrate })
            } else {
                None
            }
        })
        .collect();

    // Generate dehydrated submodule query calls
    let dehydrated_query_calls: Vec<_> = fields_info.iter().filter_map(|f| {
        if let FieldKind::Submodule = f.kind {
            let ident = &f.ident;
            let ident_indices = syn::Ident::new(&format!("{}_indices", ident), ident.span());
            Some(quote! {
                let #ident_indices = self.#ident.execute_dehydrated(driver, context, key, config, results);
            })
        } else {
            None
        }
    }).collect();

    // Generate the iproduct over submodule indices (the variable names for the Vec<u32>)
    let submodule_indices_vecs: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let ident_indices = syn::Ident::new(&format!("{}_indices", ident), ident.span());
                Some(ident_indices)
            } else {
                None
            }
        })
        .collect();

    // Generate the binding pattern for iproduct tuple (the _idx names that bind references)
    let submodule_idx_bindings: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let ident_idx = syn::Ident::new(&format!("{}_idx", ident), ident.span());
                Some(ident_idx)
            } else {
                None
            }
        })
        .collect();

    // Submodule fields with their indices
    let dehydrated_submodule_fields: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let ident_idx = syn::Ident::new(&format!("{}_idx", ident), ident.span());
                let name = f.ident.to_string();
                Some(quote! { .with_submodule(#name, *#ident_idx) })
            } else {
                None
            }
        })
        .collect();

    let search_dehydrate_impl = if has_submodules {
        quote! {
            impl #spec_impl_generics ::svql_query::session::SearchDehydrate for #search_type
            where
                #(#submodule_search_dehydrate_bounds),*
            {
                const MATCH_SCHEMA: ::svql_query::session::QuerySchema = ::svql_query::session::QuerySchema::new(
                    #struct_name_str,
                    &[ #(#wire_field_descs),* ],
                    &[ #(#submodule_field_descs),* ],
                );

                fn execute_dehydrated(
                    &self,
                    driver: &::svql_query::prelude::Driver,
                    context: &::svql_query::prelude::Context,
                    key: &::svql_query::prelude::DriverKey,
                    config: &::svql_query::prelude::Config,
                    results: &mut ::svql_query::session::DehydratedResults,
                ) -> Vec<u32> {
                    // 1. Execute submodule searches (dehydrated)
                    #(#dehydrated_query_calls)*

                    // 2. Cartesian product over submodule indices, validate, and store
                    let _haystack_index = context.get(key).unwrap().index();

                    ::svql_query::itertools::iproduct!( #(#submodule_indices_vecs.iter()),* )
                        .filter_map(|( #(#submodule_idx_bindings),* )| {
                            // TODO: Topology validation using cell IDs from dehydrated rows
                            // For now, we skip validation (will be added in follow-up)

                            // Create the composite row
                            let row = ::svql_query::session::DehydratedRow::new(self.path.to_string())
                                #(#dehydrated_submodule_fields)*;

                            Some(results.push(#struct_name_str, row))
                        })
                        .collect()
                }
            }
        }
    } else {
        // No submodules - simpler case, but composites without submodules are unusual
        quote! {
            impl #spec_impl_generics ::svql_query::session::SearchDehydrate for #search_type #spec_where_clause {
                const MATCH_SCHEMA: ::svql_query::session::QuerySchema = ::svql_query::session::QuerySchema::new(
                    #struct_name_str,
                    &[ #(#wire_field_descs),* ],
                    &[],
                );

                fn execute_dehydrated(
                    &self,
                    _driver: &::svql_query::prelude::Driver,
                    _context: &::svql_query::prelude::Context,
                    _key: &::svql_query::prelude::DriverKey,
                    _config: &::svql_query::prelude::Config,
                    results: &mut ::svql_query::session::DehydratedResults,
                ) -> Vec<u32> {
                    // Composite with no submodules - just create a single row
                    let row = ::svql_query::session::DehydratedRow::new(self.path.to_string());
                    vec![results.push(#struct_name_str, row)]
                }
            }
        }
    };

    let full_expanded = quote! {
        #expanded
        #dehydrate_impl
        #rehydrate_impl
        #search_dehydrate_impl
    };

    TokenStream::from(full_expanded)
}
