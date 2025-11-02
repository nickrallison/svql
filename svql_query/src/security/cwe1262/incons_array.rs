use crate::{
    Match, Search, State, WithPath,
    instance::Instance,
    primitives::prot_reg::ProtDffEntry,
    traits::{
        enum_composite::{EnumComposite, MatchedEnumComposite, SearchableEnumComposite},
        netlist::SearchableNetlist,
    },
};
use std::collections::HashMap;
use std::collections::HashSet;
use svql_common::{Config, ModuleConfig};
use svql_driver::{Context, Driver, DriverKey};

// Enum for array entry variants: uniform (same lock) vs. inconsistent (varying locks).
#[derive(Debug, Clone)]
pub enum RegArrayEntry<S>
where
    S: State,
{
    Uniform(ProtDffEntry<S>),      // All entries same protection
    Inconsistent(ProtDffEntry<S>), // Varying protection (vulnerable)
}

impl<S> EnumComposite<S> for RegArrayEntry<S> where S: State {}

impl<S> WithPath<S> for RegArrayEntry<S>
where
    S: State,
{
    fn find_port(&self, p: &Instance) -> Option<&crate::Wire<S>> {
        match self {
            RegArrayEntry::Uniform(entry) | RegArrayEntry::Inconsistent(entry) => {
                entry.find_port(p)
            }
        }
    }

    fn path(&self) -> Instance {
        match self {
            RegArrayEntry::Uniform(entry) | RegArrayEntry::Inconsistent(entry) => entry.path(),
        }
    }
}

impl<'ctx> MatchedEnumComposite<'ctx> for RegArrayEntry<Match<'ctx>> {}

impl SearchableEnumComposite for RegArrayEntry<Search> {
    type Hit<'ctx> = RegArrayEntry<Match<'ctx>>;

    fn context(
        driver: &Driver,
        config: &ModuleConfig,
    ) -> Result<Context, Box<dyn std::error::Error>> {
        ProtDffEntry::<Search>::context(driver, config)
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        let entries = ProtDffEntry::<Search>::query(haystack_key, context, path.clone(), config);

        // Group by array (assume path prefix for array name, e.g., "reg_bank[0]")
        let mut array_groups: HashMap<String, Vec<ProtDffEntry<Match<'ctx>>>> = HashMap::new();
        for entry in entries {
            let array_name = extract_array_name(&entry.path); // E.g., "reg_bank"
            array_groups.entry(array_name).or_default().push(entry);
        }

        let mut results = Vec::new();
        for (array_name, group) in array_groups {
            if group.len() < 2 {
                continue;
            } // Need array (2+ entries)

            // Extract control signals (locks) per entry
            let mut locks: HashSet<String> = HashSet::new();
            for entry in &group {
                let lock_sig = extract_lock_signal(entry); // From connections or name
                locks.insert(lock_sig);
            }

            let variant = if locks.len() > 1 {
                RegArrayEntry::Inconsistent(group[0].clone()) // Flag as inconsistent
            } else {
                RegArrayEntry::Uniform(group[0].clone()) // Uniform (secure)
            };

            // Filter keywords (e.g., "access", "prot")
            if array_name.contains("access") || array_name.contains("prot") {
                results.push(variant);
            }
        }

        // Post-filter inconsistencies
        results
            .into_iter()
            .filter(|r| match r {
                RegArrayEntry::Inconsistent(_) => true,
                _ => false,
            })
            .collect()
    }
}

// Helpers: Extract array name and lock signal (simplified; use path/name parsing)
fn extract_array_name(_path: &Instance) -> String {
    /* impl */
    "reg_bank".to_string()
}
fn extract_lock_signal(_entry: &ProtDffEntry<Match<'_>>) -> String {
    /* impl */
    "lock1".to_string()
}
