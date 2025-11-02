pub mod incons_array;

use crate::security::cwe1262::incons_array::RegArrayEntry;
use crate::traits::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::traits::enum_composite::SearchableEnumComposite;
use crate::{Connection, Match, Search, State, WithPath, instance::Instance};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

// Top-level: Inconsistent protection in reg array.
#[derive(Debug, Clone)]
pub struct Cwe1262<S>
where
    S: State,
{
    pub path: Instance,
    pub array_entry: RegArrayEntry<S>,
}

impl<S> Cwe1262<S>
where
    S: State,
{
    pub fn new(path: Instance, array_entry: RegArrayEntry<S>) -> Self {
        Self {
            path: path.clone(),
            array_entry,
        }
    }
}

impl<S> WithPath<S> for Cwe1262<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        self.array_entry.find_port(p)
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for Cwe1262<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        match &self.array_entry {
            RegArrayEntry::Uniform(entry) | RegArrayEntry::Inconsistent(entry) => {
                vec![vec![
                    // Lock must connect to we (protection)
                    Connection {
                        from: entry.a.clone(),
                        to: entry.b.clone(),
                    },
                ]]
            }
        }
    }
}

impl<'ctx> MatchedComposite<'ctx> for Cwe1262<Match<'ctx>> {}

impl SearchableComposite for Cwe1262<Search> {
    type Hit<'ctx> = Cwe1262<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        RegArrayEntry::<Search>::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        let entries = RegArrayEntry::<Search>::query(haystack_key, context, path.clone(), config);
        entries
            .into_iter()
            .filter_map(|entry| {
                let candidate = Cwe1262::new(path.clone(), entry.clone());
                if candidate.is_inconsistent() {
                    // Custom validation
                    Some(candidate)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl<'ctx> Cwe1262<Match<'ctx>> {
    pub fn is_inconsistent(&self) -> bool {
        matches!(&self.array_entry, RegArrayEntry::Inconsistent(_))
    }
}
