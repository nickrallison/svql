//! Typed reference to a row in another pattern's table.
//!
//! `Ref<T>` is a type-safe wrapper around a u32 row index, providing
//! compile-time guarantees that references point to the correct pattern type.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A typed reference to a row in another pattern's result table.
///
/// This is the successor to `ForeignKey<T>`, providing the same type-safety
/// with a clearer name that reflects its purpose.
///
/// # Type Safety
///
/// The type parameter `T` represents the pattern type that this reference
/// points to. This ensures at compile time that you can't accidentally
/// use a reference to one pattern type with a different pattern's table.
///
/// # Layout
///
/// Internally just a `u32` row index. The `PhantomData<T>` is zero-sized
/// and only exists for compile-time type checking.
#[repr(transparent)]
pub struct Ref<T> {
    index: u32,
    _marker: PhantomData<fn() -> T>,
}

// Manual impls to avoid requiring bounds on T

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
    /// Create a new reference from a row index.
    #[inline]
    #[must_use] 
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    /// Get the raw row index.
    #[inline]
    #[must_use] 
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Get the index as usize for indexing into vectors/slices.
    #[inline]
    #[must_use] 
    pub const fn as_usize(self) -> usize {
        self.index as usize
    }

    /// Create from a usize index (panics if > `u32::MAX` in debug).
    #[inline]
    #[must_use] 
    pub fn from_usize(index: usize) -> Self {
        debug_assert!(u32::try_from(index).is_ok(), "Ref index overflow");
        Self::new(index as u32)
    }

    /// Cast this reference to a different pattern type.
    ///
    /// # Safety
    ///
    /// This is type-safe but semantically unsafe - only use when you know
    /// the underlying index is valid for type `U`'s table.
    #[inline]
    #[must_use] 
    pub const fn cast<U>(self) -> Ref<U> {
        Ref::new(self.index)
    }
}

impl<T> fmt::Debug for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Include type name in debug output for clarity
        let type_name = std::any::type_name::<T>();
        // Shorten the type name to just the final component
        let short_name = type_name.rsplit("::").next().unwrap_or(type_name);
        write!(f, "Ref<{}>({})", short_name, self.index)
    }
}

impl<T> fmt::Display for Ref<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.index)
    }
}

impl<T> From<u32> for Ref<T> {
    #[inline]
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

impl<T> From<Ref<T>> for u32 {
    #[inline]
    fn from(r: Ref<T>) -> Self {
        r.index
    }
}

impl<T> Default for Ref<T> {
    /// Default to index 0 (first row).
    #[inline]
    fn default() -> Self {
        Self::new(0)
    }
}

// Polars integration: allow collecting Ref<T> into a column
impl<T> From<Ref<T>> for i64 {
    #[inline]
    fn from(r: Ref<T>) -> Self {
        Self::from(r.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Dummy types for testing
    struct PatternA;
    struct PatternB;

    #[test]
    fn test_ref_creation() {
        let r: Ref<PatternA> = Ref::new(42);
        assert_eq!(r.index(), 42);
        assert_eq!(r.as_usize(), 42);
    }

    #[test]
    fn test_ref_from_usize() {
        let r: Ref<PatternA> = Ref::from_usize(100);
        assert_eq!(r.index(), 100);
    }

    #[test]
    fn test_ref_type_safety() {
        // These are different types at compile time
        let _a: Ref<PatternA> = Ref::new(1);
        let _b: Ref<PatternB> = Ref::new(1);
        // They can't be mixed without explicit cast
    }

    #[test]
    fn test_ref_cast() {
        let a: Ref<PatternA> = Ref::new(5);
        let b: Ref<PatternB> = a.cast();
        assert_eq!(a.index(), b.index());
    }

    #[test]
    fn test_ref_debug() {
        let r: Ref<PatternA> = Ref::new(42);
        let debug = format!("{:?}", r);
        assert!(debug.contains("Ref"));
        assert!(debug.contains("42"));
        assert!(debug.contains("PatternA"));
    }

    #[test]
    fn test_ref_display() {
        let r: Ref<PatternA> = Ref::new(42);
        assert_eq!(format!("{}", r), "#42");
    }

    #[test]
    fn test_ref_conversions() {
        let r: Ref<PatternA> = 10u32.into();
        let back: u32 = r.into();
        assert_eq!(back, 10);
    }

    #[test]
    fn test_ref_hash() {
        use std::collections::HashSet;
        let mut set: HashSet<Ref<PatternA>> = HashSet::new();
        set.insert(Ref::new(1));
        set.insert(Ref::new(2));
        set.insert(Ref::new(1)); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_ref_ordering() {
        let a: Ref<PatternA> = Ref::new(1);
        let b: Ref<PatternA> = Ref::new(2);
        assert!(a < b);
        assert!(b > a);
    }
}
