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

    // Generics
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    // Specialized generics (S removed)
    let specialized_generics = common::remove_state_generic(&item_struct.generics);
    let (spec_impl_generics, _, spec_where_clause) = specialized_generics.split_for_impl();

    // Concrete types
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
            #ident: ::svql_query::traits::netlist::bind_match_wire(
                self.#ident.path().clone(),
                assignment,
                &assignments,
                needle,
                #wire_name
            )
        }
    });

    let find_port_arms = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        let wire_name = &f.wire_name;
        quote! {
            #wire_name => self.#ident.find_port_inner(tail)
        }
    });

    let report_logic = parsed_fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            if let Some(loc) = self.#ident.inner.as_ref().and_then(|c| c.get_source()) {
                file_path = loc.file;
                for line in loc.lines {
                    if seen.insert(line.number) {
                        all_lines.push(line);
                    }
                }
            }
        }
    });

    let expanded = quote! {
        #[derive(Clone, Debug)]
        pub struct #struct_name #impl_generics #where_clause {
            pub path: ::svql_query::instance::Instance,
            #(#struct_fields),*
        }

        impl #spec_impl_generics ::svql_query::traits::Projected for #search_type #spec_where_clause {
            type Pattern = #search_type;
            type Result = #match_type;
        }

        impl #spec_impl_generics ::svql_query::traits::Projected for #match_type #spec_where_clause {
            type Pattern = #search_type;
            type Result = #match_type;
        }

        impl #impl_generics ::svql_query::traits::Component<S> for #struct_name #ty_generics #where_clause {
            fn path(&self) -> &::svql_query::instance::Instance {
                &self.path
            }

            fn type_name(&self) -> &'static str {
                stringify!(#struct_name)
            }

            fn find_port(&self, path: &::svql_query::instance::Instance) -> Option<&::svql_query::Wire<S>> {
                if !path.starts_with(self.path()) { return None; }
                let rel_path = path.relative(self.path());
                self.find_port_inner(rel_path)
            }

            fn find_port_inner(&self, rel_path: &[std::sync::Arc<str>]) -> Option<&::svql_query::Wire<S>> {
                let next = match rel_path.first() {
                    Some(arc_str) => arc_str.as_ref(),
                    None => return None,
                };
                let tail = &rel_path[1..];
                match next {
                    #(#find_port_arms),*,
                    _ => None,
                }
            }
        }

        impl #spec_impl_generics ::svql_query::traits::Searchable for #search_type #spec_where_clause {
            fn instantiate(base_path: ::svql_query::instance::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    #(#init_fields),*
                }
            }
        }

        impl #spec_impl_generics ::svql_query::traits::Reportable for #match_type #spec_where_clause {
            fn to_report(&self, name: &str) -> ::svql_query::report::ReportNode {
                use ::svql_query::subgraph::cell::SourceLocation;

                let mut all_lines = Vec::new();
                let mut file_path = std::sync::Arc::from("");
                let mut seen = std::collections::HashSet::new();

                #(#report_logic)*

                all_lines.sort_by_key(|l| l.number);

                ::svql_query::report::ReportNode {
                    name: name.to_string(),
                    type_name: stringify!(#struct_name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc: if file_path.is_empty() { None } else { Some(SourceLocation { file: file_path, lines: all_lines }) },
                    children: Vec::new(),
                }
            }
        }

        impl #spec_impl_generics ::svql_query::traits::netlist::NetlistMeta for #search_type #spec_where_clause {
            const MODULE_NAME: &'static str = #module_name;
            const FILE_PATH: &'static str = #file_path;
            const PORTS: &'static [::svql_query::traits::netlist::PortSpec] = &[];
        }

        impl #spec_impl_generics #search_type #spec_where_clause {
            pub fn new(path: ::svql_query::instance::Instance) -> Self {
                <Self as ::svql_query::traits::Searchable>::instantiate(path)
            }
        }

        impl #spec_impl_generics ::svql_query::traits::Query for #search_type #spec_where_clause {
            fn query<'a>(
                &self,
                driver: &::svql_query::driver::Driver,
                context: &'a ::svql_query::driver::Context,
                key: &::svql_query::driver::DriverKey,
                config: &::svql_query::common::Config
            ) -> Vec<Self::Result> {
                use ::svql_query::traits::{Component, netlist::NetlistMeta};
                ::svql_query::tracing::info!("{} searching netlist", self.log_label());

                let needle_key = Self::driver_key();
                let needle_container = context.get(&needle_key)
                    .expect("Pattern design not found in context")
                    .as_ref();
                let needle = needle_container.design(); // FIX: Define needle variable
                let haystack_container = context.get(key)
                    .expect("Haystack design not found in context")
                    .as_ref();

                let assignments = ::svql_query::subgraph::SubgraphMatcher::enumerate_with_indices(
                    needle,
                    haystack_container.design(),
                    needle_container.index(),
                    haystack_container.index(),
                    needle_key.module_name().to_string(),
                    key.module_name().to_string(),
                    config,
                );

                let results: Vec<_> = assignments.items.iter().map(|assignment| {
                    #struct_name {
                        path: self.path.clone(),
                        #(#match_fields),*
                    }
                }).collect();

                ::svql_query::tracing::info!("{} found {} matches", self.log_label(), results.len());
                results
            }

            fn context(
                driver: &::svql_query::driver::Driver,
                options: &::svql_query::common::ModuleConfig
            ) -> Result<::svql_query::driver::Context, Box<dyn std::error::Error>> {
                use ::svql_query::traits::netlist::NetlistMeta;
                let key = Self::driver_key();
                let (_, design) = driver.get_or_load_design(Self::FILE_PATH, Self::MODULE_NAME, options)?;
                Ok(::svql_query::driver::Context::from_single(key.clone(), design))
            }
        }
    };

    TokenStream::from(expanded)
}
