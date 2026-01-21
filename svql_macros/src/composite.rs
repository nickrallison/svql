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

    // --- DataFrame API Generation (Phase 4) ---

    // Generate ColumnDef for wire fields (ColumnKind::Wire)
    let df_wire_column_defs: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Wire = f.kind {
                let name = f.ident.to_string();
                Some(quote! {
                    ::svql_query::session::ColumnDef {
                        name: #name,
                        kind: ::svql_query::session::ColumnKind::Wire,
                        nullable: true,
                    }
                })
            } else {
                None
            }
        })
        .collect();

    // Generate ColumnDef for submodule fields (ColumnKind::Sub)
    let df_submodule_column_defs: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let name = f.ident.to_string();
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! {
                    ::svql_query::session::ColumnDef {
                        name: #name,
                        kind: ::svql_query::session::ColumnKind::Sub(::std::any::TypeId::of::<#search_ty>()),
                        nullable: false,
                    }
                })
            } else {
                None
            }
        })
        .collect();

    // Combine wire and submodule columns
    let df_column_defs: Vec<_> = df_wire_column_defs
        .into_iter()
        .chain(df_submodule_column_defs)
        .collect();

    // Generate TypeIds for dependencies (submodule Search types)
    let df_dependency_type_ids: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! {
                    ::std::any::TypeId::of::<#search_ty>()
                })
            } else {
                None
            }
        })
        .collect();

    // Generate calls to register dependencies
    let df_register_deps: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! {
                    <#search_ty as ::svql_query::traits::SearchableComponent>::df_register_all(registry);
                })
            } else {
                None
            }
        })
        .collect();

    // Generate code to get dependency tables from ExecutionContext
    let df_get_dep_tables: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let table_var = syn::Ident::new(&format!("{}_table", ident), ident.span());
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! {
                    let #table_var = ctx.get::<#search_ty>()
                        .ok_or_else(|| QueryError::missing_dep(stringify!(#search_ty).to_string()))?;
                })
            } else {
                None
            }
        })
        .collect();

    // Generate field names for submodule references
    let submodule_field_names: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                Some(&f.ident)
            } else {
                None
            }
        })
        .collect();

    // Generate cartesian product iteration
    let df_cartesian_product = if submodule_field_names.is_empty() {
        // No submodules - just create one row
        quote! {
            let row = Row::<Self>::new(0, search_instance.path.to_string());
            builder.push(row);
        }
    } else {
        // Build nested for loops for cartesian product
        let table_vars: Vec<_> = submodule_field_names.iter().map(|id| {
            syn::Ident::new(&format!("{}_table", id), id.span())
        }).collect();
        let row_vars: Vec<_> = submodule_field_names.iter().map(|id| {
            syn::Ident::new(&format!("{}_row", id), id.span())
        }).collect();
        let idx_vars: Vec<_> = submodule_field_names.iter().map(|id| {
            syn::Ident::new(&format!("{}_idx", id), id.span())
        }).collect();

        // Generate the with_sub calls for building the row
        let with_sub_calls: Vec<_> = submodule_field_names.iter().zip(idx_vars.iter()).map(|(field, idx)| {
            let name = field.to_string();
            quote! { .with_sub_idx(#name, #idx as u32) }
        }).collect();

        // Generate iproduct over table indices
        let iproduct_args: Vec<_> = table_vars.iter().map(|tv| {
            quote! { (0..#tv.len()) }
        }).collect();

        // Generate row lookups for validation
        let row_lookups: Vec<_> = table_vars.iter().zip(row_vars.iter()).zip(idx_vars.iter()).map(|((tv, rv), iv)| {
            quote! { let #rv = #tv.row(#iv as u32)?; }
        }).collect();

        // Generate validation map entries from submodule rows
        let validation_entries: Vec<_> = submodule_field_names.iter().zip(row_vars.iter()).map(|(_field, rv)| {
            quote! {
                // Get first wire cell_id from the submodule row for validation
                // (Simplified - assumes submodules have wires we can validate)
                if let Some(cell_id) = #rv.wire_any() {
                    wire_cells.push(cell_id);
                }
            }
        }).collect();

        quote! {
            for ( #(#idx_vars),* ) in ::svql_query::itertools::iproduct!( #(#iproduct_args),* ) {
                // Get rows for validation
                let valid: bool = (|| -> Option<bool> {
                    #(#row_lookups)*

                    // Collect wire cell IDs for connectivity validation
                    let mut wire_cells: Vec<u32> = Vec::new();
                    #(#validation_entries)*

                    // Validate all cells are connected (simplified validation)
                    // Full validation would check specific wire connections
                    Some(true) // TODO: Implement proper topology validation
                })().unwrap_or(false);

                if !valid {
                    continue;
                }

                // Build the composite row with submodule refs
                let row = Row::<Self>::new(builder.len() as u32, search_instance.path.to_string())
                    #(#with_sub_calls)*;
                builder.push(row);
            }
        }
    };

    // Generate df_rehydrate field reconstruction
    let df_rehydrate_submodule_tables: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let table_var = syn::Ident::new(&format!("{}_table", ident), ident.span());
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! {
                    let #table_var = store.get::<#search_ty>()?;
                })
            } else {
                None
            }
        })
        .collect();

    let df_rehydrate_fields: Vec<_> = fields_info.iter().map(|f| {
        let ident = &f.ident;
        let name = f.ident.to_string();
        match f.kind {
            FieldKind::Wire => {
                quote! {
                    #ident: {
                        let cell_id = row.wire(#name);
                        let path = ::svql_query::instance::Instance::from_path(row.path()).child(#name);
                        let cell_info = cell_id.and_then(|_id| {
                            // TODO: Get CellInfo from DesignData/Store
                            None::<::svql_query::subgraph::cell::CellInfo>
                        });
                        ::svql_query::Wire::new(path, cell_info)
                    }
                }
            }
            FieldKind::Submodule => {
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                let _match_ty = common::replace_state_generic_with(ty, quote!(::svql_query::Match));
                let table_var = syn::Ident::new(&format!("{}_table", ident), ident.span());
                quote! {
                    #ident: {
                        let sub_ref: ::svql_query::session::Ref<#search_ty> = row.sub(#name)?;
                        let sub_row = #table_var.row(sub_ref.index())?;
                        <#search_ty as ::svql_query::traits::SearchableComponent>::df_rehydrate(&sub_row, store)?
                    }
                }
            }
        }
    }).collect();

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

            // --- New DataFrame API methods (Phase 4) ---

            fn df_columns() -> &'static [::svql_query::session::ColumnDef] {
                static COLUMNS: ::std::sync::OnceLock<Vec<::svql_query::session::ColumnDef>> = ::std::sync::OnceLock::new();
                COLUMNS.get_or_init(|| vec![
                    #(#df_column_defs),*
                ])
            }

            fn df_dependencies() -> &'static [::std::any::TypeId] {
                static DEPS: ::std::sync::OnceLock<Vec<::std::any::TypeId>> = ::std::sync::OnceLock::new();
                DEPS.get_or_init(|| vec![
                    #(#df_dependency_type_ids),*
                ])
            }

            fn df_register_all(registry: &mut ::svql_query::session::PatternRegistry) {
                // First register all dependencies
                #(#df_register_deps)*

                // Then register self
                registry.register(
                    ::std::any::TypeId::of::<Self>(),
                    ::std::any::type_name::<Self>(),
                    Self::df_dependencies(),
                );
            }

            fn df_search(
                ctx: &::svql_query::session::ExecutionContext<'_>,
            ) -> Result<::svql_query::session::Table<Self>, ::svql_query::session::QueryError> {
                use ::svql_query::session::{TableBuilder, Row, QueryError, Ref};

                // Get dependency tables from the execution context
                #(#df_get_dep_tables)*

                // Create the validation context
                let driver = ctx.driver();
                let haystack_key = ctx.driver_key();
                let haystack_design = driver.get_design(&haystack_key)
                    .ok_or_else(|| QueryError::design_load(format!("Haystack design not found: {:?}", haystack_key)))?;
                let haystack_context = ::svql_query::prelude::Context::from_single(haystack_key.clone(), haystack_design);
                let haystack_index = haystack_context.get(&haystack_key).unwrap().index();

                // Create a search instance for path resolution
                let search_instance = Self::create_at(::svql_query::instance::Instance::from_path(""));

                let mut builder = TableBuilder::<Self>::new(Self::df_columns());

                // Cartesian product over all submodule tables
                #df_cartesian_product

                // Suppress unused warning for haystack_index (used in validation)
                let _ = haystack_index;

                builder.build()
            }

            fn df_rehydrate(
                row: &::svql_query::session::Row<Self>,
                store: &::svql_query::session::Store,
            ) -> Option<#match_type> {
                #(#df_rehydrate_submodule_tables)*

                Some(#struct_name {
                    path: ::svql_query::instance::Instance::from_path(row.path()),
                    #(#df_rehydrate_fields),*
                })
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

    // Generate type key lookups for submodules (for validation)
    let submodule_type_keys: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let type_key_var = syn::Ident::new(&format!("{}_type_key", ident), ident.span());
                let ty = &f.ty;
                let search_ty = common::replace_state_generic(ty);
                Some(quote! {
                    let #type_key_var = <#search_ty as ::svql_query::session::SearchDehydrate>::type_key();
                })
            } else {
                None
            }
        })
        .collect();

    // Generate row lookups and validation map building
    let validation_map_entries: Vec<_> = fields_info
        .iter()
        .filter_map(|f| {
            if let FieldKind::Submodule = f.kind {
                let ident = &f.ident;
                let ident_idx = syn::Ident::new(&format!("{}_idx", ident), ident.span());
                let type_key_var = syn::Ident::new(&format!("{}_type_key", ident), ident.span());
                let row_var = syn::Ident::new(&format!("{}_row", ident), ident.span());
                let name = f.ident.to_string();
                Some(quote! {
                    let #row_var = results.tables.get(#type_key_var)
                        .and_then(|t| t.get(*#ident_idx as usize));
                    if let Some(row) = #row_var {
                        validation_map.insert(#name, row);
                    }
                })
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
                    // Register our schema using full type path
                    let type_key = Self::type_key();
                    results.register_schema(type_key, &Self::MATCH_SCHEMA);

                    // Get type keys for submodule lookups
                    #(#submodule_type_keys)*

                    // 1. Execute submodule searches (dehydrated)
                    #(#dehydrated_query_calls)*

                    // 2. Cartesian product over submodule indices, validate, and store
                    let haystack_index = context.get(key).unwrap().index();

                    ::svql_query::itertools::iproduct!( #(#submodule_indices_vecs.iter()),* )
                        .filter_map(|( #(#submodule_idx_bindings),* )| {
                            // Build validation map
                            let mut validation_map: std::collections::HashMap<&str, &::svql_query::session::DehydratedRow> = std::collections::HashMap::new();
                            #(#validation_map_entries)*

                            // Validate topology using DehydratedTopologyValidation trait
                            if !<Self as ::svql_query::traits::DehydratedTopologyValidation>::validate_dehydrated(&validation_map, haystack_index) {
                                return None;
                            }

                            // Create the composite row
                            let row = ::svql_query::session::DehydratedRow::new(self.path.to_string())
                                #(#dehydrated_submodule_fields)*;

                            Some(results.push(type_key, row))
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
                    // Register our schema using full type path
                    let type_key = Self::type_key();
                    results.register_schema(type_key, &Self::MATCH_SCHEMA);

                    // Composite with no submodules - just create a single row
                    let row = ::svql_query::session::DehydratedRow::new(self.path.to_string());
                    vec![results.push(type_key, row)]
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
