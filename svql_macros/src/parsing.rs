// svql_macros/src/parsing.rs

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Expr, ExprArray, ExprLit, Ident, Lit, MetaNameValue, Result};

/// A path selector like ["and1", "y"]
#[derive(Debug, Clone)]
pub struct PathSelector {
    pub segments: Vec<String>,
}

impl PathSelector {
    /// Parse from an array expression like ["and1", "y"]
    pub fn from_expr_array(arr: &ExprArray) -> Result<Self> {
        let mut segments = Vec::new();
        for elem in &arr.elems {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) = elem
            {
                segments.push(s.value());
            } else {
                return Err(Error::new_spanned(elem, "Expected string literal in path"));
            }
        }
        Ok(PathSelector { segments })
    }

    /// Generate code for a static Selector
    pub fn to_selector_tokens(&self) -> TokenStream {
        let segments: Vec<_> = self.segments.iter().map(|s| quote! { #s }).collect();
        quote! {
            svql_query::selector::Selector::static_path(&[#(#segments),*])
        }
    }
}

/// Direction for ports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

impl Direction {
    pub fn to_tokens(&self) -> TokenStream {
        match self {
            Direction::Input => quote! { svql_query::session::PortDirection::Input },
            Direction::Output => quote! { svql_query::session::PortDirection::Output },
            Direction::Inout => quote! { svql_query::session::PortDirection::Inout },
        }
    }

    pub fn to_port_constructor(&self) -> TokenStream {
        match self {
            Direction::Input => quote! { svql_query::session::Port::input },
            Direction::Output => quote! { svql_query::session::Port::output },
            Direction::Inout => quote! { svql_query::session::Port::inout },
        }
    }
}

/// Parse a direction identifier (input, output, inout)
pub fn parse_direction(ident: &Ident) -> Result<Direction> {
    match ident.to_string().as_str() {
        "input" => Ok(Direction::Input),
        "output" => Ok(Direction::Output),
        "inout" => Ok(Direction::Inout),
        other => Err(Error::new_spanned(
            ident,
            format!("Expected 'input', 'output', or 'inout', got '{}'", other),
        )),
    }
}

/// Parse a key-value pair like `key = "value"` from meta
pub fn get_string_value(nv: &MetaNameValue) -> Result<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = &nv.value
    {
        Ok(s.value())
    } else {
        Err(Error::new_spanned(&nv.value, "Expected string literal"))
    }
}

/// Parse an array expression from a meta name-value
pub fn get_array_expr(nv: &MetaNameValue) -> Result<ExprArray> {
    if let Expr::Array(arr) = &nv.value {
        Ok(arr.clone())
    } else {
        Err(Error::new_spanned(&nv.value, "Expected array expression"))
    }
}

/// Parse a nested array of arrays like [["a", "b"], ["c", "d"]]
pub fn parse_nested_paths(arr: &ExprArray) -> Result<Vec<PathSelector>> {
    let mut paths = Vec::new();
    for elem in &arr.elems {
        if let Expr::Array(inner) = elem {
            paths.push(PathSelector::from_expr_array(inner)?);
        } else {
            return Err(Error::new_spanned(elem, "Expected nested array"));
        }
    }
    Ok(paths)
}

/// Helper to extract attributes by name from a list of attributes
pub fn find_attr<'a>(attrs: &'a [syn::Attribute], name: &str) -> Option<&'a syn::Attribute> {
    attrs.iter().find(|a| a.path().is_ident(name))
}

/// Helper to extract all attributes by name
pub fn find_all_attrs<'a>(attrs: &'a [syn::Attribute], name: &str) -> Vec<&'a syn::Attribute> {
    attrs.iter().filter(|a| a.path().is_ident(name)).collect()
}
