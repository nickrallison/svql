//! Storage container for designs and their structural indices.

use ouroboros::self_referencing;
use prjunnamed_netlist::Design;
use svql_subgraph::index::graph_index::GraphIndex;

/// A self-referencing container that pairs a netlist with its graph index.
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
    /// Constructs a new container and builds the graph index for the provided design.
    #[must_use] 
    pub fn build(design: Design) -> Self {
        DesignContainerBuilder {
            design,
            index_builder: |design: &Design| GraphIndex::build(design),
        }
        .build()
    }

    /// Returns a reference to the underlying netlist design.
    #[must_use] 
    pub fn design(&self) -> &Design {
        self.borrow_design()
    }

    /// Returns a reference to the structural graph index.
    #[must_use] 
    pub fn index(&self) -> &GraphIndex<'_> {
        self.borrow_index()
    }
}
