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
        match p.dir {
            Direction::In => quote! { #id : crate::ports::InPort   ::new(stringify!(#id)) },
            Direction::Out => quote! { #id : crate::ports::OutPort  ::new(stringify!(#id)) },
            Direction::InOut => quote! { #id : crate::ports::InOutPort::new(stringify!(#id)) },
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
        quote! { #id : crate::module::lookup(&port_map, stringify!(#id)).expect(concat!("Port '", stringify!(#id), "' not found")) }
    });

    let find_arms = ir.ports.iter().map(|p| {
        let id = &p.ident;
        quote! { stringify!(#id) => Some(&self.#id) }
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::lower::{Port, Direction};
    use proc_macro2::Span;
    use syn::{Ident, Visibility, VisPublic};

    #[test]
    fn test_codegen_fields() {
        // Create a fake Ir with two ports
        let ir = Ir {
            vis: Visibility::Public(VisPublic { pub_token: Default::default() }),
            iface_ident: Ident::new("Foo", Span::call_site()),
            result_ident: Ident::new("FooResult", Span::call_site()),
            file_path: "path/to/file.v".to_string(),
            module_name: "foo_mod".to_string(),
            ports: vec![
                Port { orig_name: "a".into(), ident: Ident::new("a", Span::call_site()), dir: Direction::In },
                Port { orig_name: "b".into(), ident: Ident::new("b", Span::call_site()), dir: Direction::Out },
            ],
        };
        let ts = codegen(ir);
        let code = ts.to_string();
        // Should contain struct definitions and field names
    
        assert!(code.contains("struct Foo"));
        assert!(code.contains("pub a : crate :: ports :: InPort"));
        assert!(code.contains("pub b : crate :: ports :: OutPort"));
        // Check result struct and lookup
        assert!(code.contains("struct FooResult"));
        assert!(code.contains("pub a : svql_common :: matches :: IdString"));
        assert!(code.contains("pub b : svql_common :: matches :: IdString"));
    }
}
