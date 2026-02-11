//! Storage container pairing a netlist design with its structural graph index.
//!
//! A `DesignContainer` uses self-referencing to ensure the graph index
//! and netlist remain consistent throughout their lifetimes.

#![allow(clippy::future_not_send)]

use ouroboros::self_referencing;
use prjunnamed_netlist::Design;
use svql_common::GraphIndex;

/// Self-referencing container for a design and its graph index.
///
/// The design and index are kept together to ensure consistency and enable
/// efficient traversal during subgraph matching queries.
#[self_referencing]
pub struct DesignContainer {
    /// The parsed netlist design
    design: Design,
    /// Structural graph index for efficient matching queries
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
    /// Creates a new container and builds the graph index for the design.
    ///
    /// The index is automatically constructed from the provided design.
    #[must_use]
    pub fn build(design: Design) -> Self {
        DesignContainerBuilder {
            design,
            index_builder: |design: &Design| GraphIndex::build(design),
        }
        .build()
    }

    /// Returns a reference to the netlist design.
    #[must_use]
    pub fn design(&self) -> &Design {
        self.borrow_design()
    }

    /// Returns a reference to the graph index for pattern matching.
    #[must_use]
    pub fn index(&self) -> &GraphIndex<'_> {
        self.borrow_index()
    }
}
