//! Shared parsing utilities for SVQL procedural macros.
//!
//! Provides helpers for extracting attributes, handling path selectors,
//! and validating DSL syntax during compilation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Expr, ExprArray, ExprLit, Lit, MetaNameValue, Result};

/// A path selector like `["and1", "y"]`
#[derive(Debug, Clone)]
pub struct PathSelector {
    /// The individual components of the hierarchical path.
    pub segments: Vec<String>,
}

impl PathSelector {
    /// Parse from an array expression like `["and1", "y"]`
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
        Ok(Self { segments })
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
    /// Signal flows into the component.
    Input,
    /// Signal flows out of the component.
    Output,
    /// Bidirectional signal.
    Inout,
}

impl Direction {
    /// Generates code that calls the appropriate `Port` constructor for this direction.
    pub fn as_port_constructor(self) -> TokenStream {
        match self {
            Self::Input => quote! { svql_query::session::Port::input },
            Self::Output => quote! { svql_query::session::Port::output },
            Self::Inout => quote! { svql_query::session::Port::inout },
        }
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

/// Parse a nested array of arrays like `[["a", "b"], ["c", "d"]]`
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
