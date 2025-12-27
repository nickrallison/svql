use crate::common;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, Lit, parse_macro_input};

pub fn netlist_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let args_map = common::parse_args_map(args);

    let file_path = args_map
        .get("file")
        .expect("netlist attribute requires 'file' argument");
    let module_name = args_map
        .get("name")
        .unwrap_or(&item_struct.ident.to_string())
        .clone();

    let struct_name = &item_struct.ident;
    let generics = &item_struct.generics;
    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    // 1. Inject 'path' field if not present
    if let Fields::Named(ref mut fields) = item_struct.fields {
        let _has_path = fields
            .named
            .iter()
            .any(|f| f.ident.as_ref().map(|i| i == "path").unwrap_or(false));
    }

    // 2. Analyze Fields
    let mut field_names = Vec::new();
    let mut field_strs = Vec::new();
    let mut field_inits = Vec::new();
    let mut field_matches = Vec::new();
    let mut child_refs = Vec::new();
    let mut find_port_arms = Vec::new();
    let mut field_defs = Vec::new();

    let mut schema_cols = Vec::new();
    let mut col_indices = Vec::new();
    let mut reconstruct_fields = Vec::new();

    if let Fields::Named(ref mut fields) = item_struct.fields {
        for (idx, field) in fields.named.iter_mut().enumerate() {
            let ident = field.ident.as_ref().unwrap();
            let name_str = ident.to_string();
            let ty = &field.ty;
            let vis = &field.vis;

            let mut wire_name = name_str.clone();
            let mut attrs_to_remove = Vec::new();

            for (i, attr) in field.attrs.iter().enumerate() {
                if attr.path().is_ident("rename") {
                    if let Ok(lit) = attr.parse_args::<Lit>() {
                        if let Lit::Str(s) = lit {
                            wire_name = s.value();
                        }
                    }
                    attrs_to_remove.push(i);
                }
            }

            for i in attrs_to_remove.into_iter().rev() {
                field.attrs.remove(i);
            }

            field_names.push(ident);
            field_strs.push(wire_name.clone());

            field_defs.push(quote! {
                #vis #ident: #ty
            });

            field_inits.push(quote! {
                #ident: ::svql_query::Wire::new(base_path.child(#wire_name), ())
            });

            // For query() reconstruction
            field_matches.push(quote! {
                #ident: ::svql_query::Wire::new(
                    self.#ident.path().clone(),
                    ::svql_query::traits::netlist::resolve_wire(
                        embedding,
                        &embeddings,
                        needle,
                        #wire_name
                    ).expect(&format!("Wire {} not found in embedding", #wire_name))
                )
            });

            child_refs.push(quote! { &self.#ident });

            find_port_arms.push(quote! {
                #wire_name => self.#ident.find_port_inner(tail)
            });

            schema_cols.push(wire_name.clone());
            col_indices.push(quote! {
                #wire_name => Some(#idx)
            });
            reconstruct_fields.push(quote! {
                #ident: ::svql_query::Wire::new(
                    self.#ident.path().clone(),
                    cursor.next_cell()
                )
            });
        }
    }

    // 3. Generate Code
    let expanded = quote! {
        #[derive(Clone, Debug)]
        pub struct #struct_name #generics #where_clause {
            pub path: ::svql_query::instance::Instance,
            #(#field_defs),*
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

        impl ::svql_query::traits::Searchable for #struct_name<::svql_query::Search> {
            fn instantiate(base_path: ::svql_query::instance::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    #(#field_inits),*
                }
            }
        }

        impl<'a> ::svql_query::traits::Reportable for #struct_name<::svql_query::Match<'a>> {
            fn to_report(&self, name: &str) -> ::svql_query::report::ReportNode {
                use ::svql_query::svql_subgraph::cell::SourceLocation;

                let mut all_lines = Vec::new();
                let mut file_path = std::sync::Arc::from("");
                let mut seen = std::collections::HashSet::new();

                #(
                    if let Some(loc) = self.#field_names.inner.get_source() {
                        file_path = loc.file;
                        for line in loc.lines {
                            if seen.insert(line.number) {
                                all_lines.push(line);
                            }
                        }
                    }
                )*
                all_lines.sort_by_key(|l| l.number);

                ::svql_query::report::ReportNode {
                    name: name.to_string(),
                    type_name: stringify!(#struct_name).to_string(),
                    path: self.path.clone(),
                    details: None,
                    source_loc: SourceLocation { file: file_path, lines: all_lines },
                    children: Vec::new(),
                }
            }
        }

        impl ::svql_query::traits::netlist::NetlistMeta for #struct_name<::svql_query::Search> {
            const MODULE_NAME: &'static str = #module_name;
            const FILE_PATH: &'static str = #file_path;
            const PORTS: &'static [::svql_query::traits::netlist::PortSpec] = &[];
        }

        impl #struct_name<::svql_query::Search> {
            pub fn context(
                driver: &::svql_query::svql_driver::Driver,
                options: &::svql_query::svql_common::ModuleConfig
            ) -> Result<::svql_query::svql_driver::Context, Box<dyn std::error::Error>> {
                use ::svql_query::traits::netlist::NetlistMeta;
                let key = Self::driver_key();
                let (_, design) = driver.get_or_load_design(Self::FILE_PATH, Self::MODULE_NAME, options)?;
                Ok(::svql_query::svql_driver::Context::from_single(key.clone(), design))
            }

            pub fn new(path: ::svql_query::instance::Instance) -> Self {
                <Self as ::svql_query::traits::Searchable>::instantiate(path)
            }
        }

        impl ::svql_query::traits::Query for #struct_name<::svql_query::Search> {
            type Matched<'a> = #struct_name<::svql_query::Match<'a>>;

            fn query<'a>(
                &self,
                driver: &::svql_query::svql_driver::Driver,
                context: &'a ::svql_query::svql_driver::Context,
                key: &::svql_query::svql_driver::DriverKey,
                config: &::svql_query::svql_common::Config
            ) -> Vec<Self::Matched<'a>> {
                use ::svql_query::traits::netlist::NetlistMeta;

                let needle_key = Self::driver_key();
                let needle_container = context.get(&needle_key)
                    .expect("Pattern design not found in context")
                    .as_ref();
                let haystack_container = context.get(key)
                    .expect("Haystack design not found in context")
                    .as_ref();

                let needle = needle_container.design();
                let haystack = haystack_container.design();
                let needle_index = needle_container.index();
                let haystack_index = haystack_container.index();

                let embeddings = ::svql_query::svql_subgraph::SubgraphMatcher::enumerate_with_indices(
                    needle,
                    haystack,
                    needle_index,
                    haystack_index,
                    config,
                );

                embeddings.items.iter().map(|embedding| {
                    #struct_name {
                        path: self.path.clone(),
                        #(#field_matches),*
                    }
                }).collect()
            }
        }

        impl ::svql_query::traits::PlannedQuery for #struct_name<::svql_query::Search> {
            fn to_ir(&self, config: &::svql_query::svql_common::Config) -> ::svql_query::ir::LogicalPlan {
                use ::svql_query::traits::netlist::NetlistMeta;
                ::svql_query::ir::LogicalPlan::Scan {
                    key: Self::driver_key(),
                    config: config.clone(),
                    schema: self.expected_schema(),
                }
            }

            fn expected_schema(&self) -> ::svql_query::ir::Schema {
                ::svql_query::ir::Schema {
                    columns: vec![ #(#schema_cols.to_string()),* ]
                }
            }

            fn get_column_index(&self, rel_path: &[std::sync::Arc<str>]) -> Option<usize> {
                let next = match rel_path.first() {
                    Some(arc_str) => arc_str.as_ref(),
                    None => return None,
                };
                match next {
                    #(#col_indices),*,
                    _ => None
                }
            }

            fn reconstruct<'a>(&self, cursor: &mut ::svql_query::ir::ResultCursor<'a>) -> Self::Matched<'a> {
                #struct_name {
                    path: self.path.clone(),
                    #(#reconstruct_fields),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
