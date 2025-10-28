use super::analyze::Model;

#[derive(Clone)] // Explicit for clarity
pub struct SubPatternRef {
    pub field_name: syn::Ident,
    pub ty: syn::Type,
}

#[derive(Clone)] // Explicit for clarity
pub struct ConnectionRef {
    pub from_sub: syn::Ident,
    pub from_port: syn::Ident,
    pub to_sub: syn::Ident,
    pub to_port: syn::Ident,
}

pub struct Ir {
    pub name: syn::Ident,
    pub subs: Vec<SubPatternRef>,
    pub connections: Vec<Vec<ConnectionRef>>,
}

pub fn lower(model: Model) -> Ir {
    let subs = model
        .subs
        .into_iter()
        .map(|s| SubPatternRef {
            field_name: s.field_name,
            ty: s.ty,
        })
        .collect();

    let connections = model
        .connections
        .into_iter()
        .map(|group| {
            group
                .into_iter()
                .map(|c| ConnectionRef {
                    from_sub: c.from_sub,
                    from_port: c.from_port,
                    to_sub: c.to_sub,
                    to_port: c.to_port,
                })
                .collect()
        })
        .collect();

    Ir {
        name: model.name,
        subs,
        connections,
    }
}
