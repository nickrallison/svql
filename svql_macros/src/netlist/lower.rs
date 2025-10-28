use super::analyze::Model;

#[derive(Clone, PartialEq)]
pub struct PortRef {
    pub name: syn::Ident,
}

pub struct Ir {
    pub name: syn::Ident,
    pub module_name: String,
    pub file_path: String,
    pub inputs: Vec<PortRef>,
    pub outputs: Vec<PortRef>,
}

pub fn lower(model: Model) -> Ir {
    let inputs = model
        .inputs
        .into_iter()
        .map(|p| PortRef { name: p.name })
        .collect();
    let outputs = model
        .outputs
        .into_iter()
        .map(|p| PortRef { name: p.name })
        .collect();

    Ir {
        name: model.name,
        module_name: model.module_name,
        file_path: model.file_path,
        inputs,
        outputs,
    }
}
