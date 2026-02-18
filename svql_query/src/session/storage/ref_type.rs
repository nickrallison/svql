//! Typed reference to a row in another pattern's table.
//!
//! `Ref<T>` is the *only* public way to hold a row index.
//! It cannot be constructed from a raw integer outside the `storage` module,
//! and its internal index cannot be read outside the module either.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use super::row_index::RowIndex;
use svql_common::util;

/// A typed, opaque reference to a row in `Table<T>`.
///
/// # Permitted external operations
///
/// - Compare with another `Ref<T>` of the same type (`==`, `<`, etc.)
/// - Hash
/// - Debug / Display
/// - Pass to `Table::row(Ref<T>)` or `Store::resolve(Ref<T>)`
/// - Type-cast via `cast::<U>()` (semantically unsafe — use sparingly)
///
/// # Prohibited external operations
///
/// - Constructing a `Ref<T>` from a raw integer
/// - Extracting the raw integer from a `Ref<T>`
/// - Using a `Ref<T>` with a `Table<U>` without an explicit `cast()`
#[repr(transparent)]
pub struct Ref<T> {
    /// The row index in the target table. Opaque outside this module.
    index: RowIndex,
    /// Phantom data for compile-time type checking.
    _marker: PhantomData<T>,
}

// Manual impls to avoid requiring bounds on T.

impl<T> Clone for Ref<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Ref<T> {}

impl<T> PartialEq for Ref<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Ref<T> {}

impl<T> PartialOrd for Ref<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Ref<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<T> Hash for Ref<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> Ref<T> {
    /// Construct a new reference.
    ///
    /// Available within the crates session/storage module only.
    #[inline]
    pub(super) const fn new(index: RowIndex) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    /// Construct a reference from a raw index.
    /// Internal crate only — prevents external patterns from manually
    /// creating row indices which could lead to type confusion.
    #[inline]
    pub(crate) const fn from_raw(index: RowIndex) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    /// Read the raw index.
    #[inline]
    pub(crate) fn raw_index(self) -> RowIndex {
        self.index
    }

    /// Cast this reference to a different pattern type.
    ///
    /// # Safety (semantic)
    ///
    /// This is type-safe at the Rust level (no `unsafe` needed) but
    /// semantically unsound if the resulting `Ref<U>` is used with a
    /// `Table<U>` that doesn't correspond to the original table.
    /// Use only when you know the underlying index is valid for `U`'s table.
    #[inline]
    #[must_use]
    pub const fn cast<U>(self) -> Ref<U> {
        Ref {
            index: self.index,
            _marker: PhantomData,
        }
    }
}

impl<T> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = util::short_type_name(std::any::type_name::<T>());
        write!(f, "Ref<{}>({})", name, self.index)
    }
}

impl<T> fmt::Display for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use svql_common::HashSet;

    struct PatternA;
    struct PatternB;

    fn make_ref_a(n: u32) -> Ref<PatternA> {
        Ref::new(RowIndex::new(n))
    }

    #[test]
    fn test_ref_type_safety() {
        let _a: Ref<PatternA> = make_ref_a(1);
        // Ref<PatternB> cannot be obtained from make_ref_a without cast()
    }

    #[test]
    fn test_ref_cast() {
        let a: Ref<PatternA> = make_ref_a(5);
        let b: Ref<PatternB> = a.cast();
        // Internal indices are equal
        assert_eq!(a.raw_index(), b.raw_index());
    }

    #[test]
    fn test_ref_debug() {
        let r = make_ref_a(42);
        let s = format!("{:?}", r);
        assert!(s.contains("Ref"));
        assert!(s.contains("42"));
        assert!(s.contains("PatternA"));
    }

    #[test]
    fn test_ref_display() {
        assert_eq!(format!("{}", make_ref_a(42)), "#42");
    }

    #[test]
    fn test_ref_hash() {
        let mut set: HashSet<Ref<PatternA>> = HashSet::default();
        set.insert(make_ref_a(1));
        set.insert(make_ref_a(2));
        set.insert(make_ref_a(1)); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_ref_ordering() {
        let a = make_ref_a(1);
        let b = make_ref_a(2);
        assert!(a < b);
        assert!(b > a);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    struct DummyTypeA;
    struct DummyTypeB;

    #[derive(Clone, Debug)]
    struct ArbitraryRef(Ref<DummyTypeA>);

    impl Arbitrary for ArbitraryRef {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(Ref::new(RowIndex::new(u32::arbitrary(g))))
        }
    }

    quickcheck! {
        fn prop_ref_cast_preserves_index(r: ArbitraryRef) -> bool {
            let casted: Ref<DummyTypeB> = r.0.cast();
            casted.raw_index() == r.0.raw_index()
        }
    }
}
