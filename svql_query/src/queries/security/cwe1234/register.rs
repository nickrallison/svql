use crate::{Match, Search, State, Wire, WithPath, instance::Instance};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::cell::{CellKind, CellWrapper};

/// Basic DFF wrapper - matches any DFF cell type in the design
#[derive(Debug, Clone)]
pub struct BasicDff<S>
where
    S: State,
{
    pub path: Instance,
    pub cell: Wire<S>,
}

impl<S> BasicDff<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            cell: Wire::new(path.child("cell".to_string())),
        }
    }
}

impl BasicDff<Match<'_>> {
    pub fn with_cell(path: Instance, cell: CellWrapper<'_>) -> BasicDff<Match<'_>> {
        BasicDff {
            path: path.clone(),
            cell: Wire::with_val(
                path.child("cell".to_string()),
                Match {
                    pat_node_ref: None,
                    design_node_ref: Some(cell),
                },
            ),
        }
    }
}

impl<S> WithPath<S> for BasicDff<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        if p.height() < self.path.height() {
            return None;
        }

        let item = p.get_item(p.height())?;
        let self_name = self.path.get_item(self.path.height())?;

        if item == self_name {
            Some(&self.cell)
        } else {
            None
        }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

// Implement basic query for BasicDff (directly searches by cell type)
impl BasicDff<Search> {
    pub fn context(
        _driver: &Driver,
        _config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        Ok(Context::new())
    }

    pub fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        _config: &Config,
    ) -> Vec<BasicDff<Match<'ctx>>> {
        let haystack_container = context
            .get(haystack_key)
            .expect("Haystack design not found in context");

        let haystack_index = haystack_container.index();

        haystack_index
            .cells_topo()
            .iter()
            .filter(|cell| cell.cell_type() == CellKind::Dff)
            .map(|dff_cell| BasicDff::with_cell(path.clone(), dff_cell.clone()))
            .collect()
    }
}

/// Enum composite for any register type
///
/// Currently supports:
/// - BasicDff: Any DFF cell type
///
/// Future variants could include:
/// - AsyncDff: Async reset DFF with specific structure
/// - SyncDff: Sync reset DFF with specific structure
/// - EnabledDff: DFF with explicit enable logic
#[derive(Debug, Clone)]
pub enum RegisterAny<S>
where
    S: State,
{
    BasicDff(BasicDff<S>),
}

impl<S> WithPath<S> for RegisterAny<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        match self {
            RegisterAny::BasicDff(inner) => inner.find_port(p),
        }
    }

    fn path(&self) -> Instance {
        match self {
            RegisterAny::BasicDff(inner) => inner.path(),
        }
    }
}

impl RegisterAny<Search> {
    pub fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        BasicDff::<Search>::context(driver, config)
    }

    pub fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<RegisterAny<Match<'ctx>>> {
        tracing::info!("RegisterAny::query: searching for register cells");

        let basic_dffs = BasicDff::<Search>::query(
            haystack_key,
            context,
            path.child("basic_dff".to_string()),
            config,
        );

        tracing::info!("RegisterAny::query: Found {} basic DFFs", basic_dffs.len());

        // Convert to enum variants
        basic_dffs.into_iter().map(RegisterAny::BasicDff).collect()
    }
}
