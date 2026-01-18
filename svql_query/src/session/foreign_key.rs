//! Type-safe foreign key references between query result tables.
//!
//! A `ForeignKey<T>` is a zero-cost wrapper around `u32` that carries compile-time
//! information about which table to look up when resolving the reference.

use std::marker::PhantomData;

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
    /// Uses the type's schema to determine which table to look up.
    pub fn resolve<'a>(&self, store: &'a ResultStore) -> Option<MatchRow> {
        store.get_row(T::SCHEMA.type_name, self.index)
    }

    /// Returns the type name of the target table.
    pub const fn type_name() -> &'static str {
        T::SCHEMA.type_name
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

/// Trait for types that can be the target of a foreign key.
///
/// This is automatically implemented for types that implement `Dehydrate`.
pub trait ForeignKeyTarget {
    /// The table name where rows of this type are stored.
    const TABLE_NAME: &'static str;
}

impl<T: Dehydrate> ForeignKeyTarget for T {
    const TABLE_NAME: &'static str = T::SCHEMA.type_name;
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
