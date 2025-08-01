use quote::format_ident;
use syn::Ident;

use super::analyze::Model;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    InOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Port {
    pub orig_name: String,
    pub ident: Ident,
    pub dir: Direction,
}

pub struct Ir {
    pub vis: syn::Visibility,
    pub iface_ident: Ident,
    pub result_ident: Ident,
    pub file_path: String,
    pub module_name: String,
    pub ports: Vec<Port>,
}

/// Turn a Verilog port name into a *valid* Rust identifier.
fn sanitize(name: &str) -> Ident {
    let mut s = String::with_capacity(name.len());

    for c in name.chars().filter(|c| *c != '\\') {
        s.push(if c.is_ascii_alphanumeric() { c } else { '_' });
    }
    if s.chars().next().unwrap().is_ascii_digit() {
        s.insert(0, '_');
    }
    format_ident!("{}", s)
}

pub fn lower(model: Model) -> Ir {
    let mut ports = Vec::new();

    for p in &model.pattern.in_ports {
        ports.push(Port {
            orig_name: p.clone(),
            ident: sanitize(p),
            dir: Direction::In,
        });
    }
    for p in &model.pattern.out_ports {
        ports.push(Port {
            orig_name: p.clone(),
            ident: sanitize(p),
            dir: Direction::Out,
        });
    }
    for p in &model.pattern.inout_ports {
        ports.push(Port {
            orig_name: p.clone(),
            ident: sanitize(p),
            dir: Direction::InOut,
        });
    }

    Ir {
        vis: model.vis,
        iface_ident: model.iface_ident.clone(),
        result_ident: format_ident!("{}Result", model.iface_ident),
        file_path: model.file_path,
        module_name: model.module_name,
        ports,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_sanitize() {
    //     assert_eq!(sanitize("\\valid_name"), format_ident!("valid_name"));
    //     assert_eq!(sanitize("invalid-name"), format_ident!("invalid_name"));
    //     assert_eq!(sanitize("123invalid"), format_ident!("_123invalid"));
    // }
}
