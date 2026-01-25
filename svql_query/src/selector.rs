#[derive(Debug, Clone, Copy)]
pub struct Selector {
    pub path: &'static [&'static str],
}

impl Selector {
    pub const fn new(path: &'static [&'static str]) -> Self {
        Self { path }
    }

    pub fn head(&self) -> Option<&'static str> {
        self.path.first().copied()
    }

    pub fn tail(&self) -> Self {
        if self.path.is_empty() {
            Self { path: &[] }
        } else {
            Self {
                path: &self.path[1..],
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Endpoint {
    pub selector: Selector,
}
