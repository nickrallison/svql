use crate::{
    Connection, Match, Search, State, WithPath,
    composite::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
};
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

use super::register::RegisterAny;

/// Represents a locked register in CWE1234:
/// - A register (DFF) with an enable signal that stores protected data
/// - Its enable signal should be controlled by unlock logic
///
/// This is just a wrapper around RegisterAny that adds semantic meaning
/// for the CWE-1234 pattern (it's the register being protected).
#[derive(Debug, Clone)]
pub struct LockedRegister<S>
where
    S: State,
{
    pub path: Instance,
    pub register: RegisterAny<S>,
}

impl<S> LockedRegister<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            register: RegisterAny::AsyncEnable(super::register::AsyncDffEnable::new(
                path.child("register".to_string()),
            )),
        }
    }
}

impl<S> WithPath<S> for LockedRegister<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
            Some("register") => self.register.find_port(p),
            _ => None,
        }
    }

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for LockedRegister<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // No internal connections - just wraps a register
        vec![]
    }
}

impl<'ctx> MatchedComposite<'ctx> for LockedRegister<Match<'ctx>> {}

impl<S> LockedRegister<S>
where
    S: State,
{
    /// Get the enable wire for connection validation
    /// This is what should connect to the unlock logic output
    pub fn enable_wire(&self) -> &crate::Wire<S> {
        self.register.enable_wire()
    }

    /// Get a description of the register type for reporting
    pub fn register_type(&self) -> String {
        self.register.register_type()
    }
}

impl SearchableComposite for LockedRegister<Search> {
    type Hit<'ctx> = LockedRegister<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        RegisterAny::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        tracing::info!("LockedRegister::query: starting locked register search");

        let registers = RegisterAny::query(
            haystack_key,
            context,
            path.child("register".to_string()),
            config,
        );

        tracing::info!("LockedRegister::query: Found {} registers", registers.len());

        registers
            .into_iter()
            .map(|register| LockedRegister {
                path: path.clone(),
                register,
            })
            .collect()
    }
}
