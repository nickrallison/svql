//! Variant pattern support for zero-copy enumeration of sub-types.
//!
//! Variants don't store their own data—they reference rows in sub-type tables.
//! `VariantRef<V>` points directly into sub-tables, enabling zero-copy iteration.

use std::any::TypeId;
use std::marker::PhantomData;

use super::ref_type::Ref;
use super::store::Store;

/// A reference to a variant match, pointing into a sub-type's table.
///
/// Unlike `Ref<T>` which always points to a specific pattern table,
/// `VariantRef<V>` can point to any of the variant's sub-types.
///
/// # Example
///
/// ```ignore
/// for vref in store.iter_variant::<DffVariant<Search>>() {
///     match vref.resolve(&store).unwrap() {
///         DffVariant::Sdffe(m) => { ... }
///         DffVariant::Sdff(m) => { ... }
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct VariantRef<V> {
    /// The TypeId of the sub-type this reference points to.
    sub_type: TypeId,
    /// The row index in the sub-type's table.
    idx: u32,
    /// Phantom data for the variant type.
    _marker: PhantomData<fn() -> V>,
}

impl<V> VariantRef<V> {
    /// Create a new variant reference.
    pub fn new(sub_type: TypeId, idx: u32) -> Self {
        Self {
            sub_type,
            idx,
            _marker: PhantomData,
        }
    }

    /// Get the sub-type's TypeId.
    pub fn sub_type(&self) -> TypeId {
        self.sub_type
    }

    /// Get the row index in the sub-type's table.
    pub fn index(&self) -> u32 {
        self.idx
    }

    /// Get a typed reference to the underlying row.
    ///
    /// This returns `Ref<SubType>` which can be used to access the actual row.
    /// The caller must know the sub-type to use this properly.
    pub fn as_ref<SubType>(&self) -> Option<Ref<SubType>>
    where
        SubType: 'static,
    {
        if self.sub_type == TypeId::of::<SubType>() {
            Some(Ref::new(self.idx))
        } else {
            None
        }
    }
}

impl<V> std::fmt::Debug for VariantRef<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VariantRef")
            .field("sub_type", &self.sub_type)
            .field("idx", &self.idx)
            .finish()
    }
}

// Manual impls to avoid bounds on V

impl<V> PartialEq for VariantRef<V> {
    fn eq(&self, other: &Self) -> bool {
        self.sub_type == other.sub_type && self.idx == other.idx
    }
}

impl<V> Eq for VariantRef<V> {}

impl<V> std::hash::Hash for VariantRef<V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sub_type.hash(state);
        self.idx.hash(state);
    }
}

/// Trait for variant (enum) patterns that can resolve to different sub-types.
///
/// Variants don't execute their own search—they aggregate results from
/// their sub-types. The `register_all()` method registers sub-types only.
///
/// # Generated Code
///
/// For a variant like:
/// ```ignore
/// #[variant(ports(clk, d, q))]
/// pub enum DffVariant<S: State> {
///     Sdffe(Sdffe<S>),
///     Sdff(Sdff<S>),
///     Adff(Adff<S>),
/// }
/// ```
///
/// The macro generates:
/// ```ignore
/// impl VariantPattern for DffVariant<Search> {
///     const SUB_TYPES: &'static [TypeId] = &[
///         TypeId::of::<Sdffe<Search>>(),
///         TypeId::of::<Sdff<Search>>(),
///         TypeId::of::<Adff<Search>>(),
///     ];
///
///     fn resolve_variant(sub_type: TypeId, idx: u32, store: &Store) -> Option<Self::Match> {
///         if sub_type == TypeId::of::<Sdffe<Search>>() {
///             let row = store.get::<Sdffe<Search>>()?.row(idx)?;
///             Some(DffVariant::Sdffe(Sdffe::rehydrate(&row, store)?))
///         } else if sub_type == TypeId::of::<Sdff<Search>>() {
///             // ...
///         }
///         // ...
///     }
/// }
/// ```
pub trait VariantPattern: Sized + 'static {
    /// The Match type for this variant pattern.
    type Match;

    /// TypeIds of all sub-types that this variant can hold.
    const SUB_TYPES: &'static [TypeId];

    /// Resolve a variant reference to the concrete Match type.
    ///
    /// Given the sub-type's TypeId and row index, rehydrate the appropriate
    /// sub-type from the store and wrap it in the variant's Match type.
    fn resolve_variant(sub_type: TypeId, idx: u32, store: &Store) -> Option<Self::Match>;

    /// Register all sub-types into the registry.
    ///
    /// Variants don't have their own search—they just ensure all sub-types
    /// are registered.
    fn register_sub_types(registry: &mut super::registry::PatternRegistry);
}

/// Extension trait for Store to enable variant iteration.
pub trait StoreVariantExt {
    /// Iterate over all matches of a variant's sub-types.
    ///
    /// Returns `VariantRef<V>` for each row in each sub-type's table.
    fn iter_variant<V: VariantPattern>(&self) -> VariantIter<'_, V>;
}

impl StoreVariantExt for Store {
    fn iter_variant<V: VariantPattern>(&self) -> VariantIter<'_, V> {
        VariantIter::new(self)
    }
}

/// Iterator over variant references from multiple sub-type tables.
pub struct VariantIter<'a, V> {
    store: &'a Store,
    /// Index into SUB_TYPES
    sub_type_idx: usize,
    /// Current row index in current sub-type's table
    row_idx: u32,
    /// Length of current sub-type's table
    current_len: usize,
    _marker: PhantomData<V>,
}

impl<'a, V: VariantPattern> VariantIter<'a, V> {
    fn new(store: &'a Store) -> Self {
        let mut iter = Self {
            store,
            sub_type_idx: 0,
            row_idx: 0,
            current_len: 0,
            _marker: PhantomData,
        };
        // Initialize to first non-empty sub-type table
        iter.advance_to_next_table();
        iter
    }

    /// Move to the next sub-type table that exists and has rows.
    fn advance_to_next_table(&mut self) {
        while self.sub_type_idx < V::SUB_TYPES.len() {
            let type_id = V::SUB_TYPES[self.sub_type_idx];
            if let Some(table) = self.store.get_any(type_id)
                && !table.is_empty() {
                    self.current_len = table.len();
                    self.row_idx = 0;
                    return;
                }
            self.sub_type_idx += 1;
        }
        // No more tables
        self.current_len = 0;
    }
}

impl<'a, V: VariantPattern> Iterator for VariantIter<'a, V> {
    type Item = VariantRef<V>;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we've exhausted all sub-types
        if self.sub_type_idx >= V::SUB_TYPES.len() {
            return None;
        }

        // If we've exhausted the current table, move to the next
        while self.row_idx as usize >= self.current_len {
            self.sub_type_idx += 1;
            if self.sub_type_idx >= V::SUB_TYPES.len() {
                return None;
            }
            self.advance_to_next_table();
        }

        // Return reference to current row
        let type_id = V::SUB_TYPES[self.sub_type_idx];
        let idx = self.row_idx;
        self.row_idx += 1;
        Some(VariantRef::new(type_id, idx))
    }
}

/// Helper to resolve a VariantRef to the Match type.
impl<V: VariantPattern> VariantRef<V> {
    /// Resolve this reference to the concrete Match type.
    pub fn resolve(&self, store: &Store) -> Option<V::Match> {
        V::resolve_variant(self.sub_type, self.idx, store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SubTypeA;
    struct SubTypeB;

    #[test]
    fn test_variant_ref_creation() {
        let vref: VariantRef<()> = VariantRef::new(TypeId::of::<SubTypeA>(), 42);
        assert_eq!(vref.sub_type(), TypeId::of::<SubTypeA>());
        assert_eq!(vref.index(), 42);
    }

    #[test]
    fn test_variant_ref_as_ref() {
        let vref: VariantRef<()> = VariantRef::new(TypeId::of::<SubTypeA>(), 10);
        
        // Correct type
        let r: Option<Ref<SubTypeA>> = vref.as_ref();
        assert!(r.is_some());
        assert_eq!(r.unwrap().index(), 10);
        
        // Wrong type
        let r: Option<Ref<SubTypeB>> = vref.as_ref();
        assert!(r.is_none());
    }

    #[test]
    fn test_variant_ref_equality() {
        let a: VariantRef<()> = VariantRef::new(TypeId::of::<SubTypeA>(), 1);
        let b: VariantRef<()> = VariantRef::new(TypeId::of::<SubTypeA>(), 1);
        let c: VariantRef<()> = VariantRef::new(TypeId::of::<SubTypeA>(), 2);
        let d: VariantRef<()> = VariantRef::new(TypeId::of::<SubTypeB>(), 1);
        
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }
}
