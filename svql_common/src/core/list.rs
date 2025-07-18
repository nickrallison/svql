#[repr(C)]
pub struct List<T> {
    pub ptr: *mut T,
    pub len: usize,
    pub cap: usize,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Vec::new().into()
    }

    pub fn append(&mut self, item: T) {
        with_vec(self, |vec| vec.push(item));
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        if self.cap != 0 && !self.ptr.is_null() {
            unsafe {
                let _ = Vec::from_raw_parts(self.ptr, self.len, self.cap);
            }
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self { Self::new() }
}

impl<T> std::iter::FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Vec::from_iter(iter).into()
    }
}

impl<T> std::iter::Extend<T> for List<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        with_vec(self, |vec| vec.extend(iter));
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        let me = std::mem::ManuallyDrop::new(self);
        unsafe { 
            Vec::from_raw_parts(me.ptr, me.len, me.cap).into_iter()
        }
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(mut vec: Vec<T>) -> Self {
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        let cap = vec.capacity();
        std::mem::forget(vec);
        List { ptr, len, cap }
    }
}

impl<T> Into<Vec<T>> for List<T> {
    fn into(self) -> Vec<T> {
        let me = std::mem::ManuallyDrop::new(self);
        unsafe { Vec::from_raw_parts(me.ptr, me.len, me.cap) }
    }
}

fn with_vec<T, F, R>(list: &mut List<T>, f: F) -> R
where
    F: FnOnce(&mut Vec<T>) -> R,
{
    let old = std::mem::take(list);
    let mut vec: Vec<T> = unsafe {
        let old = std::mem::ManuallyDrop::new(old);
        Vec::from_raw_parts(old.ptr, old.len, old.cap)
    };
    let result = f(&mut vec);
    *list = List::from(vec);
    result
}

impl<T> std::ops::Deref for List<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> AsRef<[T]> for List<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T> AsMut<[T]> for List<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_slice().fmt(f)
    }
}
impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl<T: Eq> Eq for List<T> {}
impl<T: PartialOrd> PartialOrd for List<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}
impl<T: Ord> Ord for List<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}
impl<T: std::hash::Hash> std::hash::Hash for List<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        Self::from(self.as_slice().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_and_empty() {
        let list: List<i32> = List::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        assert_eq!(list.as_slice(), &[0; 0]);
    }

    #[test]
    fn test_default() {
        let list: List<i32> = List::default();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
        assert_eq!(list.as_slice(), &[0; 0]);
    }

    #[test]
    fn test_append() {
        let mut list = List::new();
        list.append(10);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        assert_eq!(list.as_slice(), &[10]);

        list.append(20);
        assert_eq!(list.len(), 2);
        assert_eq!(list.as_slice(), &[10, 20]);

        // Test appending to trigger reallocation
        let mut list2 = List::new();
        for i in 0..100 {
            list2.append(i);
        }
        assert_eq!(list2.len(), 100);
        for i in 0..100 {
            assert_eq!(list2.as_slice()[i], i);
        }
    }

    #[test]
    fn test_as_mut_slice() {
        let mut list = List::from(vec![1, 2, 3]);
        let slice = list.as_mut_slice();
        slice[1] = 99;
        assert_eq!(list.as_slice(), &[1, 99, 3]);
    }

    #[test]
    fn test_from_vec() {
        let mut vec = vec![1, 2, 3, 4];
        let vec_ptr = vec.as_mut_ptr();
        let vec_len = vec.len();
        let vec_cap = vec.capacity();

        let list = List::from(vec);
        assert_eq!(list.ptr, vec_ptr);
        assert_eq!(list.len, vec_len);
        assert_eq!(list.cap, vec_cap);
        assert_eq!(list.as_slice(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_into_vec() {
        let mut list = List::new();
        list.append(1);
        list.append(2);

        let list_ptr = list.ptr;
        let list_len = list.len;
        let list_cap = list.cap;

        let vec: Vec<i32> = list.into();
        assert_eq!(vec.as_ptr(), list_ptr);
        assert_eq!(vec.len(), list_len);
        assert_eq!(vec.capacity(), list_cap);
        assert_eq!(vec, vec![1, 2]);
    }

    #[test]
    fn test_drop() {
        // This test primarily checks that the Drop implementation doesn't panic.
        // Running tests with Miri is the best way to detect memory leaks or double frees.
        let list = List::from(vec![1, 2, 3]);
        drop(list);

        // Test dropping an empty list created with new()
        let empty_list: List<i32> = List::new();
        drop(empty_list);

        // Test dropping an empty list created from an empty vec
        let empty_list_from_vec = List::from(Vec::<i32>::new());
        drop(empty_list_from_vec);
    }

    #[test]
    fn test_from_iter() {
        let data = [1, 2, 3, 4, 5];
        let list: List<_> = data.iter().copied().collect();
        assert_eq!(list.len(), 5);
        assert_eq!(list.as_slice(), &data);

        let empty_list: List<i32> = [].iter().copied().collect();
        assert!(empty_list.is_empty());
    }

    #[test]
    fn test_extend() {
        let mut list = List::from(vec![1, 2]);
        list.extend([3, 4, 5].iter().copied());
        assert_eq!(list.as_slice(), &[1, 2, 3, 4, 5]);

        // Test extending with an empty iterator
        let original_len = list.len();
        list.extend([].iter().copied());
        assert_eq!(list.len(), original_len);

        // Test extending an empty list
        let mut empty_list: List<i32> = List::new();
        empty_list.extend(vec![10, 20]);
        assert_eq!(empty_list.as_slice(), &[10, 20]);
    }

    #[test]
    fn test_into_iter() {
        let list = List::from(vec![10, 20, 30]);
        let mut collected = Vec::new();
        for item in list {
            collected.push(item);
        }
        assert_eq!(collected, vec![10, 20, 30]);

        // Test on empty list
        let empty_list: List<i32> = List::new();
        assert_eq!(empty_list.into_iter().next(), None);
    }

    #[test]
    fn test_deref() {
        let list = List::from(vec![10, 20, 30]);
        // Methods from slice should be available through Deref
        assert_eq!(list.first(), Some(&10));
        assert!(list.contains(&20));
        assert!(!list.contains(&40));
    }

    #[test]
    fn test_as_ref_as_mut() {
        let mut list = List::from(vec![1, 2, 3]);

        let as_ref_slice: &[i32] = list.as_ref();
        assert_eq!(as_ref_slice, &[1, 2, 3]);

        let as_mut_slice: &mut [i32] = list.as_mut();
        as_mut_slice[0] = 100;
        assert_eq!(list.as_slice(), &[100, 2, 3]);
    }

    #[test]
    fn test_debug_format() {
        let list = List::from(vec![1, 2, 3]);
        let formatted = format!("{:?}", list);
        assert_eq!(formatted, "[1, 2, 3]");

        let empty_list: List<i32> = List::new();
        let empty_formatted = format!("{:?}", empty_list);
        assert_eq!(empty_formatted, "[]");
    }

    #[test]
    fn test_equality() {
        let list1 = List::from(vec![1, 2, 3]);
        let list2 = List::from(vec![1, 2, 3]);
        let list3 = List::from(vec![1, 2, 4]);
        let list4 = List::from(vec![1, 2]);

        assert_eq!(list1, list2);
        assert_ne!(list1, list3);
        assert_ne!(list1, list4);

        let empty1: List<i32> = List::new();
        let empty2: List<i32> = List::new();
        assert_eq!(empty1, empty2);
        assert_ne!(empty1, list1);
    }

    #[test]
    fn test_ordering() {
        let list1 = List::from(vec![1, 2, 3]);
        let list2 = List::from(vec![1, 2, 4]);
        let list3 = List::from(vec![1, 2, 3]);

        assert!(list1 < list2);
        assert!(list2 > list1);
        assert!(list1 <= list3);
        assert!(list1 >= list3);
    }

    #[test]
    fn test_hash() {
        let list1 = List::from(vec![1, 2, 3]);
        let list2 = List::from(vec![1, 2, 3]);
        let list3 = List::from(vec![3, 2, 1]);

        let mut set = HashSet::new();
        set.insert(list1);

        // An identical list should already be in the set
        assert!(set.contains(&list2));

        // A different list should not be in the set
        assert!(!set.contains(&list3));
        set.insert(list3);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_clone() {
        let list1 = List::from(vec![10, 20, 30]);
        let mut list2 = list1.clone();

        // The clone should be equal
        assert_eq!(list1, list2);
        // But they should not point to the same memory
        assert_ne!(list1.ptr, list2.ptr);

        // Modifying the clone should not affect the original
        list2.append(40);
        assert_eq!(list1.as_slice(), &[10, 20, 30]);
        assert_eq!(list2.as_slice(), &[10, 20, 30, 40]);

        // Test cloning an empty list
        let empty1: List<i32> = List::new();
        let empty2 = empty1.clone();
        assert_eq!(empty1, empty2);
        assert!(empty2.is_empty());
    }

    #[test]
    fn test_zero_sized_types() {
        // Vec (and thus List) has special handling for ZSTs.
        // The pointer is dangling, but capacity is usize::MAX.
        let mut list: List<()> = List::new();
        assert_eq!(list.len(), 0);
        assert!(list.cap > 0); // Capacity is non-zero for ZSTs

        list.append(());
        list.append(());
        list.append(());
        assert_eq!(list.len(), 3);

        let list2 = list.clone();
        assert_eq!(list2.len(), 3);

        let collected: List<()> = std::iter::repeat(()).take(5).collect();
        assert_eq!(collected.len(), 5);

        // Ensure drop doesn't panic
        drop(list);
        drop(list2);
        drop(collected);
    }
}