#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Selector<'a> {
    path: &'a [&'a str],
}

impl<'a> Selector<'a> {
    pub const fn new(path: &'a [&'a str]) -> Self {
        Self { path }
    }

    pub const fn head(&self) -> Option<&'a str> {
        self.path.first().copied()
    }

    pub fn tail(&self) -> Self {
        match self.path.len() {
            0 | 1 => Self { path: &[] },
            _ => Self {
                path: &self.path[1..],
            },
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub const fn len(&self) -> usize {
        self.path.len()
    }

    pub const fn path(&self) -> &[&'a str] {
        self.path
    }
}

impl Selector<'static> {
    pub const fn static_path(path: &'static [&'static str]) -> Self {
        Self { path }
    }
}
