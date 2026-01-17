//! Variant component traits and utilities.
//!
//! Provides traits for polymorphic pattern components.

use crate::prelude::*;
use crate::traits::component::{MatchedComponent, SearchableComponent, kind};

/// Trait for variant (polymorphic) pattern components.
///
/// Implemented by types generated with `#[variant]`. Variants allow a single
/// query point to match multiple different underlying implementations.
pub trait VariantComponent: SearchableComponent<Kind = kind::Variant> {
    /// Names of the common ports exposed across all variants.
    const COMMON_PORTS: &'static [&'static str];

    /// Executes searches for all variant types and aggregates results.
    fn search_variants(
        &self,
        driver: &Driver,
        context: &Context,
        key: &DriverKey,
        config: &Config,
    ) -> Vec<Self::Match>;
}

/// Trait for the matched state of variant components.
pub trait VariantMatched: MatchedComponent {
    type SearchType: VariantComponent<Match = Self>;

    /// Returns which variant was matched.
    fn variant_name(&self) -> &'static str;
}
