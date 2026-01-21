//! Type-safe foreign key references between query result tables.
//!
//! A `ForeignKey<T>` is a zero-cost wrapper around `u32` that carries compile-time
//! information about which table to look up when resolving the reference.
//!
//! **DEPRECATED**: Use [`Ref<T>`](super::Ref) instead. `ForeignKey<T>` is an alias
//! for backwards compatibility and will be removed in a future version.

use std::marker::PhantomData;

use super::ref_type::Ref;
use super::{Dehydrate, MatchRow, ResultStore};

/// A typed foreign key reference to a row in another table.
///
/// At runtime, this is just a `u32` index. At compile time, the phantom
/// type `T` indicates which table to look up when resolving.
///
/// # Example
///
/// ```ignore
/// // Store a reference to a RecOr match
/// let rec_or_fk: ForeignKey<RecOr<Match>> = results.push_typed::<RecOr<Match>>(row);
///
/// // Later, resolve it to get the actual row
/// let rec_or_row = rec_or_fk.resolve(&store)?;
/// let or_y = rec_or_row.wire("or_y");
/// ```
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForeignKey<T> {
    index: u32,
    _marker: PhantomData<fn() -> T>, // Covariant, no drop check issues
}

impl<T> ForeignKey<T> {
    /// Creates a new foreign key from a raw index.
    #[inline]
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    /// Gets the raw index for DataFrame storage.
    #[inline]
    pub const fn raw(self) -> u32 {
        self.index
    }

    /// Gets the raw index as usize for array indexing.
    #[inline]
    pub const fn as_usize(self) -> usize {
        self.index as usize
    }
}

impl<T: Dehydrate> ForeignKey<T> {
    /// Resolves this foreign key to a row from the correct table.
    ///
    /// Uses `std::any::type_name::<T>()` to look up the correct table,
    /// which includes the full module path for unique identification.
    pub fn resolve<'a>(&self, store: &'a ResultStore) -> Option<MatchRow> {
        store.get_row(std::any::type_name::<T>(), self.index)
    }

    /// Returns the full type path of the target table.
    pub fn type_key() -> &'static str {
        std::any::type_name::<T>()
    }
}

// Allow converting raw u32 into ForeignKey
impl<T> From<u32> for ForeignKey<T> {
    #[inline]
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

// Allow converting ForeignKey into raw u32
impl<T> From<ForeignKey<T>> for u32 {
    #[inline]
    fn from(fk: ForeignKey<T>) -> Self {
        fk.raw()
    }
}

// --- Interop with Ref<T> ---

impl<T> From<Ref<T>> for ForeignKey<T> {
    #[inline]
    fn from(r: Ref<T>) -> Self {
        Self::new(r.index())
    }
}

impl<T> From<ForeignKey<T>> for Ref<T> {
    #[inline]
    fn from(fk: ForeignKey<T>) -> Self {
        Ref::new(fk.raw())
    }
}

impl<T> ForeignKey<T> {
    /// Convert to the new `Ref<T>` type.
    #[inline]
    pub fn to_ref(self) -> Ref<T> {
        Ref::new(self.index)
    }

    /// Create from a `Ref<T>`.
    #[inline]
    pub fn from_ref(r: Ref<T>) -> Self {
        Self::new(r.index())
    }
}

/// Trait for types that can be the target of a foreign key.
///
/// This is automatically implemented for types that implement `Dehydrate`.
#[deprecated(since = "0.2.0", note = "Use Pattern trait instead")]
pub trait ForeignKeyTarget {
    /// The table name where rows of this type are stored (full type path).
    fn table_name() -> &'static str;
}

#[allow(deprecated)]
impl<T: Dehydrate> ForeignKeyTarget for T {
    fn table_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_key_is_zero_cost() {
        // ForeignKey<T> should be the same size as u32
        assert_eq!(
            std::mem::size_of::<ForeignKey<()>>(),
            std::mem::size_of::<u32>()
        );
    }

    #[test]
    fn test_foreign_key_roundtrip() {
        let fk: ForeignKey<()> = ForeignKey::new(42);
        assert_eq!(fk.raw(), 42);
        assert_eq!(fk.as_usize(), 42);

        let fk2: ForeignKey<()> = 42.into();
        assert_eq!(fk, fk2);

        let raw: u32 = fk.into();
        assert_eq!(raw, 42);
    }
}
