use super::analyze::Model;

#[derive(Clone)]
pub struct VariantRef {
    pub variant_name: syn::Ident,
    pub inst_name: String,
    pub ty: syn::Type,
}

pub struct Ir {
    pub name: syn::Ident,
    pub variants: Vec<VariantRef>,
}

pub fn lower(model: Model) -> Ir {
    let variants = model
        .variants
        .into_iter()
        .map(|v| VariantRef {
            variant_name: v.variant_name,
            inst_name: v.inst_name.value(),
            ty: v.ty,
        })
        .collect();

    Ir {
        name: model.name,
        variants,
    }
}
