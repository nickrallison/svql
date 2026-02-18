/// A hierarchical path selector for navigating through netlist structures.
///
/// Represents a path as a sequence of string segments, useful for
/// referring to nested modules, cells, or ports within a design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selector<'a> {
    /// The path segments.
    path: &'a [&'a str],
}

impl<'a> Selector<'a> {
    /// Creates a new selector from a path slice.
    pub const fn new(path: &'a [&'a str]) -> Self {
        Self { path }
    }

    /// Returns the first segment of the path, if any.
    pub const fn head(&self) -> Option<&'a str> {
        self.path.first().copied()
    }

    /// Returns a new selector with all segments except the first.
    pub fn tail(&self) -> Self {
        match self.path.len() {
            0 | 1 => Self { path: &[] },
            _ => Self {
                path: &self.path[1..],
            },
        }
    }

    /// Returns true if the path has no segments.
    pub const fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// Returns the number of segments in the path.
    pub const fn len(&self) -> usize {
        self.path.len()
    }

    /// Returns the underlying path slice.
    pub const fn path(&self) -> &[&'a str] {
        self.path
    }
}

impl Selector<'static> {
    /// Creates a selector with a 'static lifetime path.
    pub const fn static_path(path: &'static [&'static str]) -> Self {
        Self { path }
    }
}
