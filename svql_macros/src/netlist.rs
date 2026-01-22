use crate::common;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, Lit, parse_macro_input};

pub fn netlist_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);
    let args_map = common::parse_args_map(args);

    let file_path = args_map
        .get("file")
        .expect("netlist attribute requires 'file'");
    let module_name = args_map
        .get("name")
        .unwrap_or(&item_struct.ident.to_string())
        .clone();

    let struct_name = &item_struct.ident;
    let _struct_name_str = struct_name.to_string(); // Kept for potential future use

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

    // --- Parsing Phase ---

    struct FieldInfo {
        ident: syn::Ident,
        ty: syn::Type,
        vis: syn::Visibility,
        wire_name: String,
    }

    let mut parsed_fields = Vec::new();

    if let Fields::Named(ref fields) = item_struct.fields {
        for field in &fields.named {
            let ident = field.ident.clone().unwrap();
            if ident == "path" {
                continue;
            }

            let mut wire_name = ident.to_string();
            for attr in &field.attrs {
                if attr.path().is_ident("rename") {
                    if let Ok(Lit::Str(s)) = attr.parse_args::<Lit>() {
                        wire_name = s.value();
                    }
                }
            }

            parsed_fields.push(FieldInfo {
                ident,
                ty: field.ty.clone(),
                vis: field.vis.clone(),
                wire_name,
            });
        }
    }

    // --- Generation Phase ---

    let struct_fields = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let ty = &f.ty;
        let vis = &f.vis;
        quote! { #vis #ident: #ty }
    });

    let init_fields = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let wire_name = &f.wire_name;
        quote! {
            #ident: ::svql_query::Wire::new(base_path.child(#wire_name), ())
        }
    });

    let match_fields = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let wire_name = &f.wire_name;
        quote! {
            #ident: resolver.bind_wire(
                self.#ident.path().clone(),
                assignment,
                #wire_name
            )
        }
    });

    let children_impl = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        quote! { &self.#ident }
    });

    // --- New DataFrame API columns (Phase 4) ---
    let column_defs = parsed_fields.iter().map(|f| {
        let name = f.ident.to_string();
        quote! {
            ::svql_query::session::ColumnDef::wire(#name)
        }
    });

    let rehydrate_from_row_fields = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let name = f.ident.to_string();
        let wire_name = &f.wire_name;
        quote! {
            #ident: {
                // Construct wire from Row data - cell info lookup happens lazily via Store
                let wire_path = ::svql_query::instance::Instance::from_path(row.path()).child(#wire_name);
                let cell_id = row.wire(#name);
                // For now, we don't have cell info in the new API - just the CellId
                // The Match state requires Option<CellInfo>, so we return None
                // This can be improved later with a design lookup
                ::svql_query::Wire::matched(wire_path, None)
            }
        }
    });

    let row_wire_fields = parsed_fields.iter().map(|f| {
        let name = f.ident.to_string();
        let wire_name = &f.wire_name;
        quote! {
            .with_wire(#name, resolver.get_cell_id(assignment, #wire_name).map(|id| ::svql_query::session::CellId::new(id as u32)))
        }
    });

    let expanded = quote! {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct #struct_name #impl_generics #where_clause {
            pub path: ::svql_query::instance::Instance,
            #(#struct_fields),*
        }

        // Hardware implementation (state-generic)
        impl #impl_generics ::svql_query::traits::Hardware for #struct_name #ty_generics #where_clause {
            type State = S;

            fn path(&self) -> &::svql_query::instance::Instance { &self.path }
            fn type_name(&self) -> &'static str { stringify!(#struct_name) }

            fn children(&self) -> Vec<&dyn ::svql_query::traits::Hardware<State = Self::State>> {
                vec![ #(#children_impl),* ]
            }
        }

        // SearchableComponent implementation (Search state)
        impl #spec_impl_generics ::svql_query::traits::SearchableComponent for #search_type #spec_where_clause {
            type Kind = ::svql_query::traits::kind::Netlist;
            type Match = #match_type;

            fn create_at(base_path: ::svql_query::instance::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    #(#init_fields),*
                }
            }

            fn build_context(
                driver: &::svql_query::prelude::Driver,
                options: &::svql_query::prelude::ModuleConfig
            ) -> Result<::svql_query::driver::Context, Box<dyn std::error::Error>> {
                use ::svql_query::traits::NetlistComponent;
                let (_, design) = driver.get_or_load_design(Self::FILE_PATH, Self::MODULE_NAME, options)?;
                Ok(::svql_query::prelude::Context::from_single(Self::driver_key(), design))
            }

            fn execute_search(
                &self,
                driver: &::svql_query::prelude::Driver,
                context: &::svql_query::prelude::Context,
                key: &::svql_query::prelude::DriverKey,
                config: &::svql_query::prelude::Config
            ) -> Vec<Self::Match> {
                use ::svql_query::traits::{NetlistComponent, execute_netlist_query};
                use ::svql_query::prelude::PortResolver;

                let assignments = execute_netlist_query(self, context, key, config);
                let needle_container = context.get(&Self::driver_key()).unwrap();
                let resolver = PortResolver::new(needle_container.index());

                assignments.items.iter().map(|assignment| {
                    self.bind_match(&resolver, assignment)
                }).collect()
            }

            // --- New DataFrame API methods (Phase 4) ---

            fn df_columns() -> &'static [::svql_query::session::ColumnDef] {
                static COLUMNS: ::std::sync::OnceLock<Vec<::svql_query::session::ColumnDef>> = ::std::sync::OnceLock::new();
                COLUMNS.get_or_init(|| vec![
                    #(#column_defs),*
                ])
            }

            fn df_dependencies() -> &'static [::std::any::TypeId] {
                &[] // Netlists have no dependencies
            }

            fn df_register_all(registry: &mut ::svql_query::session::PatternRegistry) {
                registry.register(
                    ::std::any::TypeId::of::<Self>(),
                    ::std::any::type_name::<Self>(),
                    Self::df_dependencies(),
                );
            }

            fn df_register_search(registry: &mut ::svql_query::session::SearchRegistry)
            where
                Self: Send + Sync + 'static,
            {
                use ::svql_query::session::{AnyTable, SearchFn};

                // Netlists have no dependencies, so just register self
                let search_fn: SearchFn = |ctx| {
                    let table = Self::df_search(ctx)?;
                    Ok(Box::new(table) as Box<dyn AnyTable>)
                };

                registry.register(
                    ::std::any::TypeId::of::<Self>(),
                    ::std::any::type_name::<Self>(),
                    Self::df_dependencies(),
                    search_fn,
                );
            }

            fn df_search(
                _ctx: &::svql_query::session::ExecutionContext<'_>,
            ) -> Result<::svql_query::session::Table<Self>, ::svql_query::session::QueryError> {
                use ::svql_query::traits::{NetlistComponent, execute_netlist_query, Hardware, SearchableComponent};
                use ::svql_query::prelude::PortResolver;
                use ::svql_query::session::{TableBuilder, Row, QueryError};

                // Get driver from ExecutionContext
                let driver = _ctx.driver();
                let haystack_key = _ctx.driver_key();

                // Build the query context (loads needle design)
                let options = ::svql_query::prelude::ModuleConfig::default();
                let needle_context = Self::build_context(driver, &options)
                    .map_err(|e| QueryError::needle_load(e.to_string()))?;

                // Get the haystack context
                let haystack_design = driver.get_design(&haystack_key)
                    .ok_or_else(|| QueryError::design_load(format!("Haystack design not found: {:?}", haystack_key)))?;
                let haystack_context = ::svql_query::prelude::Context::from_single(haystack_key.clone(), haystack_design);

                // Merge contexts
                let full_context = needle_context.merge(haystack_context);

                // Get configuration
                let config = ::svql_common::Config::default();

                // Create a search instance at root
                let search_instance = Self::create_at(::svql_query::instance::Instance::from_path(""));
                let needle_key = Self::driver_key();

                // Execute the netlist query
                let assignments = execute_netlist_query(&search_instance, &full_context, &haystack_key, &config);

                // Early return for empty results
                if assignments.items.is_empty() {
                    return ::svql_query::session::Table::empty(Self::df_columns());
                }

                let needle_container = full_context.get(&needle_key)
                    .ok_or_else(|| QueryError::missing_dep(stringify!(#struct_name).to_string()))?;
                let resolver = PortResolver::new(needle_container.index());

                let mut builder = TableBuilder::<Self>::new(Self::df_columns());
                for assignment in &assignments.items {
                    let row = Row::<Self>::new(0, search_instance.path.to_string())
                        #(#row_wire_fields)*;
                    builder.push(row);
                }

                builder.build()
            }

            fn df_rehydrate(
                row: &::svql_query::session::Row<Self>,
                _store: &::svql_query::session::Store,
            ) -> Option<#match_type> {
                Some(#struct_name {
                    path: ::svql_query::instance::Instance::from_path(row.path()),
                    #(#rehydrate_from_row_fields),*
                })
            }
        }

        // NetlistComponent implementation (Search state)
        impl #spec_impl_generics ::svql_query::traits::NetlistComponent for #search_type #spec_where_clause {
            const MODULE_NAME: &'static str = #module_name;
            const FILE_PATH: &'static str = #file_path;

            fn bind_match(
                &self,
                resolver: &::svql_query::prelude::PortResolver,
                assignment: &::svql_query::prelude::SingleAssignment,
            ) -> Self::Match {
                #struct_name {
                    path: self.path.clone(),
                    #(#match_fields),*
                }
            }
        }

        // MatchedComponent implementation (Match state)
        impl #spec_impl_generics ::svql_query::traits::MatchedComponent for #match_type #spec_where_clause {
            type Search = #search_type;
        }

        // NetlistMatched implementation (Match state)
        impl #spec_impl_generics ::svql_query::traits::NetlistMatched for #match_type #spec_where_clause {
            type SearchType = #search_type;
        }
    };

    TokenStream::from(expanded)
}
