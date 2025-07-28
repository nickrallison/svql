//! Final phase – emit Rust code.

use proc_macro2::TokenStream;
use quote::quote;

use super::lower::{Direction, Ir};

pub fn codegen(ir: Ir) -> TokenStream {
    let vis = &ir.vis;
    let iface_ident = &ir.iface_ident;
    let result_ident = &ir.result_ident;
    let file_path = &ir.file_path;
    let module_name = &ir.module_name;

    // --------------------------- interface struct ------------------------
    let in_fields = field_tokens(&ir, Direction::In, quote! { crate::ports::InPort    });
    let out_fields = field_tokens(&ir, Direction::Out, quote! { crate::ports::OutPort   });
    let inout_fields = field_tokens(&ir, Direction::InOut, quote! { crate::ports::InOutPort });

    let new_inits = ir.ports.iter().map(|p| {
        let id = &p.ident;
        let name = &p.orig_name;
        match p.dir {
            Direction::In => quote! { #id : crate::ports::InPort   ::new(#name) },
            Direction::Out => quote! { #id : crate::ports::OutPort  ::new(#name) },
            Direction::InOut => quote! { #id : crate::ports::InOutPort::new(#name) },
        }
    });

    let init_full_path_calls = ir.ports.iter().map(|p| {
        let id = &p.ident;
        quote! { self.#id.init_full_path(full_path.clone()); }
    });

    // --------------------------- result struct ---------------------------
    let result_fields = ir.ports.iter().map(|p| {
        let id = &p.ident;
        quote! { #vis #id : svql_common::matches::IdString }
    });

    let lookup_arms = ir.ports.iter().map(|p| {
        let id   = &p.ident;
        let name = &p.orig_name;
        quote! { #id : crate::module::lookup(&port_map, #name).expect(concat!("Port '", #name, "' not found")) }
    });

    let find_arms = ir.ports.iter().map(|p| {
        let id = &p.ident;
        let name = &p.orig_name;
        quote! { #name => Some(&self.#id) }
    });

    // --------------------------- assemble -------------------------------
    quote! {
        // ================= interface ====================================
        #[derive(Debug, Clone, PartialEq, Eq)]
        #vis struct #iface_ident {
            #(#in_fields ,)*
            #(#out_fields ,)*
            #(#inout_fields ,)*
        }

        impl #iface_ident {
            #vis fn new() -> Self {
                Self { #(#new_inits ,)* }
            }
        }

        impl crate::module::traits::RtlModuleTrait for #iface_ident {
            type Result = #result_ident;

            fn file_path(&self) -> std::path::PathBuf { #file_path.into() }
            fn module_name(&self) -> &'static str { #module_name }
            fn init_full_path(&mut self, full_path: std::collections::VecDeque<std::sync::Arc<String>>) {
                #(#init_full_path_calls)*
            }
        }

        // ================= result =======================================
        #[derive(Debug, Clone, PartialEq, Eq)]
        #vis struct #result_ident {
            #(#result_fields ,)*
        }

        impl crate::module::traits::RtlModuleResultTrait for #result_ident {
            fn from_portmap(
                port_map: std::collections::HashMap<
                    svql_common::matches::IdString,
                    svql_common::matches::IdString>
            ) -> Self {
                Self { #(#lookup_arms ,)* }
            }

            fn find_port(
                &self,
                port_name: std::collections::VecDeque<std::sync::Arc<String>>
            ) -> Option<&svql_common::matches::IdString> {
                if port_name.len() != 2 { return None; }
                match port_name[1].as_str() {
                    #(#find_arms ,)*
                    _ => None,
                }
            }
        }
    }
}

fn field_tokens(ir: &Ir, dir: Direction, ty: TokenStream) -> Vec<TokenStream> {
    ir.ports
        .iter()
        .filter(|p| p.dir == dir)
        .map(|p| {
            let id = &p.ident;
            let vis = &ir.vis;
            quote! { #vis #id : #ty }
        })
        .collect()
}
