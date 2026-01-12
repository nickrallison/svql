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

            // Skip "path" if the user manually added it, as we inject it automatically now
            if ident == "path" {
                continue;
            }

            let mut kind = FieldKind::Wire; // Default
            for attr in &field.attrs {
                if attr.path().is_ident("submodule") {
                    kind = FieldKind::Submodule;
                }
                // We ignore #[path] attributes now as the field is auto-generated
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
                   #ident: <#search_ty as ::svql_query::traits::Pattern>::instantiate(base_path.child(#name_str))
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
                    let #ident = self.#ident.execute(driver, context, key, config);
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
                let sub_ctx = <#search_ty>::context(driver, options)?;
                ctx = ctx.merge(sub_ctx);
            })
        } else {
            None
        }
    });

    let expanded = quote! {
        #[derive(Clone, Debug)]
        pub struct #struct_name #impl_generics #where_clause {
            pub path: ::svql_query::instance::Instance,
            #(#struct_fields),*
        }

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

        impl #spec_impl_generics ::svql_query::prelude::Pattern for #search_type #spec_where_clause {
            type Match = #match_type;

            fn instantiate(base_path: ::svql_query::prelude::Instance) -> Self {
                Self {
                    path: base_path.clone(),
                    #(#instantiate_fields),*
                }
            }

            fn context(
                driver: &::svql_query::prelude::Driver,
                options: &::svql_query::prelude::ModuleConfig
            ) -> Result<::svql_query::prelude::Context, Box<dyn std::error::Error>> {
                let mut ctx = ::svql_query::prelude::Context::new();
                #(#context_calls)*
                Ok(ctx)
            }

            fn execute(
                &self,
                driver: &::svql_query::prelude::Driver,
                context: &::svql_query::prelude::Context,
                key: &::svql_query::prelude::DriverKey,
                config: &::svql_query::prelude::Config
            ) -> Vec<Self::Match> {
                use ::svql_query::prelude::validate_composite;

                // 1. Execute sub-queries
                #(#query_calls)*

                // 2. Cartesian Product & Filtering
                let haystack_index = context.get(key).unwrap().index();

                ::svql_query::itertools::iproduct!( #(#query_vars),* )
                    .map(|( #(#query_vars),* )| {
                        #struct_name {
                            path: self.path.clone(),
                            #(#construct_fields),*
                        }
                    })
                    .filter(|candidate| validate_composite(candidate, haystack_index))
                    .collect()
            }
        }

        impl #spec_impl_generics ::svql_query::prelude::Matched for #match_type #spec_where_clause {
            type Search = #search_type;
        }
    };

    TokenStream::from(expanded)
}
