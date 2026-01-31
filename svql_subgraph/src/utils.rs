//! Utility functions for the subgraph matcher.

use std::collections::HashSet;
use std::hash::Hash;

/// Computes the intersection of multiple sets.
/// Returns an empty set if the input is empty.
#[must_use] 
pub fn intersect_sets<T, I>(mut items: Vec<I>) -> HashSet<T>
where
    T: Eq + Hash + Clone,
    I: IntoIterator<Item = T>,
{
    let Some(first_iter) = items.pop() else {
        return HashSet::new();
    };

    let mut result: HashSet<T> = first_iter.into_iter().collect();

    for item in items {
        let other: HashSet<T> = item.into_iter().collect();
        result.retain(|x| other.contains(x));
    }

    result
}
