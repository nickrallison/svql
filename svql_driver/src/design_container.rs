use ouroboros::self_referencing;
use prjunnamed_netlist::Design;
use svql_subgraph::design_index::DesignIndex;

// #[derive(Debug)]
#[self_referencing]
pub struct DesignContainer {
    design: Design,
    #[borrows(design)]
    #[covariant]
    index: DesignIndex<'this>,
}

impl std::fmt::Debug for DesignContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DesignContainer")
            .field("design", &self.borrow_design())
            .field("index", &self.borrow_index())
            .finish()
    }
}

impl DesignContainer {
    pub fn build(design: Design) -> Self {
        DesignContainerBuilder {
            design,
            index_builder: |design: &Design| DesignIndex::build(design),
        }
        .build()
    }
    pub fn design(&self) -> &Design {
        self.borrow_design()
    }
    pub fn index(&self) -> &DesignIndex<'_> {
        self.borrow_index()
    }
}
