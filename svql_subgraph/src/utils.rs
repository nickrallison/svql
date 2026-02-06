//! Utility functions for the subgraph matcher.

use ahash::AHashSet;
use std::hash::Hash;

/// Computes the intersection of multiple sets.
/// Returns an empty set if the input is empty.
#[must_use] 
pub fn intersect_sets<T, I>(mut items: Vec<I>) -> AHashSet<T>
where
    T: Eq + Hash + Clone,
    I: IntoIterator<Item = T>,
{
    let Some(first_iter) = items.pop() else {
        return AHashSet::new();
    };

    let mut result: AHashSet<T> = first_iter.into_iter().collect();

    for item in items {
        let other: AHashSet<T> = item.into_iter().collect();
        result.retain(|x| other.contains(x));
    }

    result
}
