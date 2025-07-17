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
        let mut vec: Vec<T> = std::mem::take(self).into();
        vec.push(item);
        *self = List::from(vec);
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
                Vec::from_raw_parts(self.ptr, self.len, self.cap);
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
        let mut vec: Vec<T> = std::mem::take(self).into();
        vec.extend(iter);
        *self = List::from(vec);
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
        List {
            ptr,
            len,
            cap,
        }
    }
}

impl<T> Into<Vec<T>> for List<T> {
    fn into(self) -> Vec<T> {
        unsafe { Vec::from_raw_parts(self.ptr, self.len, self.cap) }
    }
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

impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
impl<T: Hash> Hash for List<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        Self::from(self.as_slice().to_vec())
    }
}