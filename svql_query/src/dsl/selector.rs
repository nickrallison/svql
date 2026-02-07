/// A path selector with generic lifetime for flexibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selector<'a> {
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
    #[must_use] 
    pub const fn static_path(path: &'static [&'static str]) -> Self {
        Self { path }
    }
}
