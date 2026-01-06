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

    let expanded = quote! {
        #[derive(Clone, Debug)]
        pub struct #struct_name #impl_generics #where_clause {
            pub path: ::svql_query::instance::Instance,
            #(#struct_fields),*
        }

        impl #impl_generics ::svql_query::traits::Hardware for #struct_name #ty_generics #where_clause {
            type State = S;

            fn path(&self) -> &::svql_query::instance::Instance { &self.path }
            fn type_name(&self) -> &'static str { stringify!(#struct_name) }

            fn children(&self) -> Vec<&dyn ::svql_query::traits::Hardware<State = Self::State>> {
                vec![ #(#children_impl),* ]
            }
        }

        impl #spec_impl_generics ::svql_query::traits::Pattern for #search_type #spec_where_clause {
            type Match = #match_type;

            fn instantiate(base_path: ::svql_query::instance::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    #(#init_fields),*
                }
            }

            fn context(
                driver: &::svql_query::prelude::Driver,
                options: &::svql_query::prelude::ModuleConfig
            ) -> Result<::svql_query::driver::Context, Box<dyn std::error::Error>> {
                use ::svql_query::prelude::Netlist;
                let (_, design) = driver.get_or_load_design(Self::FILE_PATH, Self::MODULE_NAME, options)?;
                Ok(::svql_query::prelude::Context::from_single(Self::driver_key(), design))
            }

            fn execute(
                &self,
                driver: &::svql_query::prelude::Driver,
                context: &::svql_query::prelude::Context,
                key: &::svql_query::prelude::DriverKey,
                config: &::svql_query::prelude::Config
            ) -> Vec<Self::Match> {
                use ::svql_query::prelude::{Netlist, execute_netlist_query};
                use ::svql_query::prelude::PortResolver;

                let assignments = execute_netlist_query(self, context, key, config);
                let needle_container = context.get(&Self::driver_key()).unwrap();
                let resolver = PortResolver::new(needle_container.index());

                assignments.items.iter().map(|assignment| {
                    #struct_name {
                        path: self.path.clone(),
                        #(#match_fields),*
                    }
                }).collect()
            }
        }

        impl #spec_impl_generics ::svql_query::traits::netlist::Netlist for #search_type #spec_where_clause {
            const MODULE_NAME: &'static str = #module_name;
            const FILE_PATH: &'static str = #file_path;
        }

        impl #spec_impl_generics ::svql_query::traits::Matched for #match_type #spec_where_clause {
            type Search = #search_type;
        }
    };

    TokenStream::from(expanded)
}
