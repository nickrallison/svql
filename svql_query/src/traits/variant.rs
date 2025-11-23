use svql_common::{Config, ModuleConfig};
use svql_driver::{DriverKey, context::Context, driver::Driver};

use crate::{Match, Search, State, instance::Instance};

pub trait Variant<S>
where
    S: State,
{
}

pub trait SearchableVariant: Variant<Search> {
    type Hit<'ctx>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>>;

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>>;
}

pub trait MatchedVariant<'ctx>: Variant<Match<'ctx>> {}
