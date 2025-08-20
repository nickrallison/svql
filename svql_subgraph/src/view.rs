use prjunnamed_netlist::Design;

/// A thin, owned wrapper around a borrowed Design, to avoid bare references
/// in public function signatures while keeping zero-overhead lifetimes.
#[derive(Clone, Copy, Debug)]
pub struct DesignView<'a> {
    pub design: &'a Design,
}

impl<'a> DesignView<'a> {
    pub fn new(design: &'a Design) -> Self {
        DesignView { design }
    }

    pub fn as_ref(&self) -> &'a Design {
        self.design
    }
}
