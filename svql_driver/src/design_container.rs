use ouroboros::self_referencing;
use prjunnamed_netlist::Design;
use svql_subgraph::graph_index::GraphIndex;

// #[derive(Debug)]
#[self_referencing]
pub struct DesignContainer {
    design: Design,
    #[borrows(design)]
    #[covariant]
    index: GraphIndex<'this>,
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
            index_builder: |design: &Design| GraphIndex::build(design),
        }
        .build()
    }
    pub fn design(&self) -> &Design {
        self.borrow_design()
    }
    pub fn index(&self) -> &GraphIndex<'_> {
        self.borrow_index()
    }
}
