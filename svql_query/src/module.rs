use std::{path::PathBuf};
use std::collections::{HashMap, HashSet};

use crate::{
    ports::{Connection},
    query::{run_svql_query_leaf},
};

use svql_common::{
    mat::{IdString, SanitizedQueryMatch},
};


use lazy_static::lazy_static;

use crate::{
    net::{SvqlQueryError},
};
use crate::ports::{InPort, OutPort};
use crate::query::{connections_satisfied, try_merge_matches};

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection<InPort, OutPort>> = HashSet::new();
}


pub trait Module {
    type Result;

    fn file_path(&self)   -> PathBuf;
    fn module_name(&self) -> String;
    fn instance_name(&self) -> &str;

    fn submodules(&self)  -> Vec<&dyn Module> { Vec::new() }
    fn connections(&self) -> &HashSet<Connection<InPort, OutPort>> { &EMPTY_CONNECTIONS }

    // public façade – starts recursion with empty prefix
    fn query_net(&self, addr: &str)
        -> Result<Self::Result, SvqlQueryError>
    {
        self.query_net_rec(addr, "")
    }

    // ------------------------------------------------------------------------
    // INTERNAL  recursive worker  (adds `prefix`)
    // ------------------------------------------------------------------------
    fn query_net_rec(
        &self,
        addr: &str,
        prefix: &str,          // already contains the trailing dot, or ""
    ) -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError>
    {
        let my_path = if prefix.is_empty() {
            // top-level:  ""        + "inst" + "."  → "inst."
            format!("{}.", self.instance_name())
        } else {
            // nested:     "parent." + "inst" + "."  → "parent.inst."
            format!("{prefix}{}.", self.instance_name())
        };

        println!("▶ query_net_rec( prefix = \"{prefix}\" inst = \"{}\" )",
                   self.instance_name());

        // ── leaf ────────────────────────────────────────────────────────────
        if self.submodules().is_empty() {
            println!("  • leaf – running SVQL");
            let mut res = run_svql_query_leaf(
                addr, self.file_path(), self.module_name()
            )?;
            println!("  • leaf – {} matches", res.len());

            // add full hierarchical prefix
            let leaf_prefix = &my_path[.. my_path.len() - 1]; // drop trailing dot
            for m in &mut res {
                m.port_map = m.port_map
                    .drain()
                    .map(|(k, v)| match k {
                        IdString::Named(name) =>
                            (IdString::Named(format!("{leaf_prefix}.{name}")), v),
                        other => (other, v),
                    })
                    .collect();
            }
            return Ok(res);
        }

        // ── composite ───────────────────────────────────────────────────────
        let mut per_child = Vec::new();
        for sm in self.submodules() {
            per_child.push(sm.query_net_rec(addr, &my_path)?);
        }

        // merge children (unchanged) …
        let mut combined = vec![SanitizedQueryMatch {
            port_map: HashMap::new(),
            cell_map: HashMap::new(),
        }];
        for child_res in per_child {
            let mut next = Vec::new();
            for base in &combined {
                for ch in &child_res {
                    if let Some(m) = try_merge_matches(base, ch) {
                        next.push(m);
                    }
                }
            }
            combined = next;
        }

        combined.retain(|m| connections_satisfied(m, self.connections()));
        Ok(combined)
    }
}