use std::collections::HashSet;
use std::hash::Hash;

pub fn intersect_sets<T: Eq + Hash + Clone>(mut items: Vec<HashSet<T>>) -> HashSet<T> {
    let Some(first_fanin) = items.pop() else {
        return HashSet::new();
    };

    let intersection: HashSet<T> = items.iter().fold(first_fanin, |acc: HashSet<T>, hs| {
        acc.intersection(hs).cloned().collect()
    });

    intersection
}

pub fn intersect_sets_ref<T: Eq + Hash + Clone>(mut items: Vec<&HashSet<T>>) -> HashSet<T> {
    let Some(first_fanin) = items.pop() else {
        return HashSet::new();
    };

    let first_fanin = first_fanin.clone();

    let intersection: HashSet<T> = items.iter().fold(first_fanin, |acc: HashSet<T>, hs| {
        acc.intersection(hs).cloned().collect()
    });

    intersection
}
