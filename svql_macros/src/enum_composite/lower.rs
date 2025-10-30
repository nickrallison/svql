// svql_macros/src/enum_composites/lower.rs
use super::analyze::Model;

#[derive(Clone)]
pub struct VariantRef {
    pub variant_name: syn::Ident,
    pub var_name: syn::Ident,
    pub inst_name: String,
    pub ty: syn::Type,
}

pub struct Ir {
    pub name: syn::Ident,
    pub variants: Vec<VariantRef>,
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

pub fn lower(model: Model) -> Ir {
    let variants = model
        .variants
        .into_iter()
        .map(|v| {
            let var_name = syn::Ident::new(
                &to_snake_case(&v.variant_name.to_string()),
                v.variant_name.span(),
            );
            VariantRef {
                variant_name: v.variant_name,
                var_name,
                inst_name: v.inst_name.value().to_string(),
                ty: v.ty,
            }
        })
        .collect();

    Ir {
        name: model.name,
        variants,
    }
}
