use svql_driver::{DriverKey, context::Context, driver::Driver};
use svql_subgraph::Config;

use crate::{Match, Search, State, instance::Instance};

pub trait EnumComposite<S>
where
    S: State,
{
}

pub trait SearchableEnumComposite: EnumComposite<Search> {
    type Hit<'ctx>;

    fn context(driver: &Driver) -> Result<Context, Box<dyn std::error::Error>>;

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>>;
}

pub trait MatchedEnumComposite<'ctx>: EnumComposite<Match<'ctx>> {}
