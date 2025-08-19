use svql_driver::prelude::Driver;
use svql_subgraph::{SubgraphMatch, find_subgraphs};

use crate::instance::Instance;

/// Direction of a port on a netlist
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortDir {
    In,
    Out,
}

/// Static description of a port (for future macro/introspection)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortSpec {
    pub name: &'static str,
    pub dir: PortDir,
}

/// Static, macro‑friendly metadata for a generated netlist type.
///
/// The implementor is typically the Search variant of a generic netlist struct
/// (e.g., `And<Search>`), but you can also implement it for a dedicated “type
/// tag” if you prefer. A derive‑macro would likely emit this impl.
pub trait NetlistMeta {
    /// Human/Verilog module name of the pattern (e.g., "and_gate")
    const MODULE_NAME: &'static str;

    /// On‑disk path for loading/compiling the pattern (for reference/tooling)
    const FILE_PATH: &'static str;

    /// Port list for simple introspection and future tooling
    const PORTS: &'static [PortSpec];
}

/// Uniform, macro‑friendly query surface for a generated netlist type.
///
/// The implementor is the `Search` variant type for the given netlist (e.g.,
/// `And<Search>`). A macro can emit this impl with the matching variant set as
/// `type Hit<'p, 'd> = And<Match<'p, 'd>>`, together with a `from_subgraph`
/// that fills each port Wire by calling the standard helpers.
///
/// Notes:
/// - The associated function `query` is static and accepts pattern + haystack
///   Drivers plus a placement Instance where the result will be rooted.
/// - The binding policy (which port name maps to input vs output and how to
///   extract bit 0, etc.) is encoded inside `from_subgraph`.
pub trait SearchableNetlist: NetlistMeta + Sized {
    /// The matched/instantiated form this netlist maps to on a concrete hit.
    type Hit<'p, 'd>;

    /// Construct a typed match from a raw `SubgraphMatch` at `path`.
    fn from_subgraph<'p, 'd>(m: &SubgraphMatch<'p, 'd>, path: Instance) -> Self::Hit<'p, 'd>;

    /// Static search entry point. The macro can emit this as part of the impl
    /// (or rely on this default). Users can call:
    /// `And::<Search>::query(pattern, haystack, Instance::root("and".into()))`.
    fn query<'p, 'd>(
        pattern: &'p Driver,
        haystack: &'d Driver,
        path: Instance,
    ) -> Vec<Self::Hit<'p, 'd>> {
        find_subgraphs(pattern.design_as_ref(), haystack.design_as_ref())
            .iter()
            .map(|m| Self::from_subgraph(m, path.clone()))
            .collect()
    }
}
