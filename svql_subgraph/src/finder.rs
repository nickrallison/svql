//! A small ergonomic wrapper around `find_subgraphs`.
//!
//! This type makes it straightforward for consumers to discover how to call
//! the search and to evolve parameters in the future (e.g., strategies,
//! pruning knobs) without breaking the top-level API.

use prjunnamed_netlist::Design;

use crate::{AllSubgraphMatches, SubgraphMatch, config::Config, find_subgraphs};

/// High-level convenience API for running a subgraph search.
///
/// Typical usage:
/// - Construct with pattern and design `Design` references
/// - Call `find_all()` or `find_first()`
///
/// Example (no_run):
/// ```no_run
/// use prjunnamed_netlist::Design;
/// use svql_subgraph::Finder;
/// use svql_subgraph::config::Config;
///
/// fn run(pattern: &Design, design: &Design) {
///     let match_length = true;
///     let config = Config::new(match_length);
///     let finder = Finder::new(pattern, design, config);
///     let results = finder.find_all();
///     for m in &results {
///         // Iterate a match’s cell mappings:
///         for (p, d) in m.iter() {
///             // p and d are CellWrapper references (pattern -> design)
///             let _ = (p.debug_index(), d.debug_index());
///         }
///     }
/// }
/// ```
pub struct Finder<'p, 'd> {
    pattern: &'p Design,
    design: &'d Design,
    config: Config,
}

impl<'p, 'd> Finder<'p, 'd> {
    /// Create a new `Finder` for a given `pattern` and `design`.
    pub fn new(pattern: &'p Design, design: &'d Design, config: Config) -> Self {
        Self {
            pattern,
            design,
            config,
        }
    }

    /// Find all subgraph matches from `pattern` into `design`.
    pub fn find_all(&self) -> AllSubgraphMatches<'p, 'd> {
        find_subgraphs(self.pattern, self.design, &self.config)
    }

    /// Find the first subgraph match from `pattern` into `design`, if any.
    pub fn find_first(&self) -> Option<SubgraphMatch<'p, 'd>> {
        let all = self.find_all();
        all.matches.into_iter().next()
    }
}
