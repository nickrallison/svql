/// A path selector with generic lifetime for flexibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selector<'a> {
    path: &'a [&'a str],
}

impl<'a> Selector<'a> {
    /// Create a selector from a path slice
    pub const fn new(path: &'a [&'a str]) -> Self {
        Self { path }
    }

    /// Get the first segment
    pub fn head(&self) -> Option<&'a str> {
        self.path.first().copied()
    }

    /// Get everything after the first segment
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
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// Get the full path
    pub fn path(&self) -> &[&'a str] {
        self.path
    }

    /// Get a specific segment by index
    pub fn segment(&self, idx: usize) -> Option<&'a str> {
        self.path.get(idx).copied()
    }
}

// Convenience for static selectors
impl Selector<'static> {
    pub const fn static_path(path: &'static [&'static str]) -> Self {
        Self { path }
    }
}
