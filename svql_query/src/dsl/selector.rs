//! Hierarchical path navigation for pattern fields.
//!
//! The `Selector` type allows queries to traverse through nested
//! submodules to target specific ports or internal signals.

/// A path selector with generic lifetime for flexibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selector<'a> {
    /// The ordered list of path segments.
    path: &'a [&'a str],
}

impl<'a> Selector<'a> {
    /// Create a selector from a path slice
    #[must_use]
    pub const fn new(path: &'a [&'a str]) -> Self {
        Self { path }
    }

    /// Get the first segment
    #[must_use]
    pub const fn head(&self) -> Option<&'a str> {
        self.path.first().copied()
    }

    /// Get everything after the first segment
    #[must_use]
    pub fn tail(&self) -> Self {
        if self.path.len() <= 1 {
            Self { path: &[] }
        } else {
            Self {
                path: &self.path[1..],
            }
        }
    }

    /// Check if empty
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// Get length
    #[must_use]
    pub const fn len(&self) -> usize {
        self.path.len()
    }

    /// Get the full path
    #[must_use]
    pub const fn path(&self) -> &[&'a str] {
        self.path
    }

    /// Get a specific segment by index
    #[must_use]
    pub fn segment(&self, idx: usize) -> Option<&'a str> {
        self.path.get(idx).copied()
    }
}

// Convenience for static selectors
impl Selector<'static> {
    /// Creates a path selector from a static slice of strings.
    #[must_use]
    pub const fn static_path(path: &'static [&'static str]) -> Self {
        Self { path }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use lazy_static::lazy_static;
    use quickcheck::{Arbitrary, Gen, quickcheck};

    #[derive(Clone, Debug)]
    struct ArbitrarySelector(Selector<'static>);

    // We need a static lifetime for the selector slices in quickcheck.
    // This is tricky because we need to generate static strings.
    // We can use a fixed set of static strings for generation.
    lazy_static! {
        static ref STATIC_STRINGS: Vec<&'static str> =
            vec!["a", "b", "c", "clk", "q", "d", "submod", "port"];
    }

    impl Arbitrary for ArbitrarySelector {
        fn arbitrary(g: &mut Gen) -> Self {
            let len = usize::arbitrary(g) % 5;
            let segments: Vec<&'static str> = (0..len)
                .map(|_| *g.choose(&STATIC_STRINGS[..]).unwrap())
                .collect();

            // Leak the vec to get 'static lifetime for the test run
            // Or just use Selector::static_path with a leaked box
            let boxed: Box<[&'static str]> = segments.into_boxed_slice();
            let static_ref: &'static [&'static str] = Box::leak(boxed);

            Self(Selector::new(static_ref))
        }
    }

    quickcheck! {
        fn prop_selector_head_tail(s: ArbitrarySelector) -> bool {
            if s.0.is_empty() {
                s.0.head().is_none() && s.0.tail().is_empty()
            } else {
                let _head = s.0.head().unwrap();
                let tail = s.0.tail();

                // Reconstruct check (conceptually)
                // Since we can't easily concat slices, check properties
                s.0.len() == tail.len() + 1
            }
        }

        fn prop_selector_len(s: ArbitrarySelector) -> bool {
            s.0.len() == s.0.path().len()
        }
    }
}
