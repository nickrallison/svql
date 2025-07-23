use std::{
    collections::{
        hash_map::{Entry},
        HashMap,
        HashSet,
    },
    net::ToSocketAddrs,
    path::PathBuf,
};

use svql_common::{
    config::ffi::SvqlRuntimeConfig,
    mat::{IdString, SanitizedQueryMatch},
};

use lazy_static::lazy_static;

use crate::{
    connection::Connection,
    net::{run_svql_query, SvqlQueryError},
};

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection> = HashSet::new();
}

// ────────────────────────────────────────────────────
// helpers (unchanged except for the pub fields)
// ────────────────────────────────────────────────────
fn try_merge_matches(
    m1: &SanitizedQueryMatch,
    m2: &SanitizedQueryMatch,
) -> Option<SanitizedQueryMatch> {

    println!("  ↳ try_merge_matches()");
    println!("    m1.port_map = {:?}", m1.port_map);
    println!("    m2.port_map = {:?}", m2.port_map);

    let mut port_map = m1.port_map.clone();
    for (k, v) in &m2.port_map {
        match port_map.entry(k.clone()) {
            Entry::Vacant(e)   => { e.insert(v.clone()); }
            Entry::Occupied(e) => if e.get() != v { return None; }
        }
    }

    let mut cell_map = m1.cell_map.clone();
    for (k, v) in &m2.cell_map {
        match cell_map.entry(k.clone()) {
            Entry::Vacant(e)   => { e.insert(v.clone()); }
            Entry::Occupied(e) => if e.get() != v { return None; }
        }
    }

    println!("    ↳ merged OK");
    println!("    port_map = {:?}", port_map);
    println!("    cell_map = {:?}", cell_map);
    Some(SanitizedQueryMatch { port_map, cell_map })
}

fn connections_satisfied(m: &SanitizedQueryMatch, conns: &HashSet<Connection>,) -> bool {

    println!("  ↳ connections_satisfied()");
    for c in conns {
        println!("    needs  {} == {}", c.in_port.0, c.out_port.0);
    }

    conns.iter().all(|c| {
        let k_lhs = IdString::Named(c.in_port.0.clone());
        let k_rhs = IdString::Named(c.out_port.0.clone());

        let res = matches!(
            (m.port_map.get(&k_lhs), m.port_map.get(&k_rhs)),
            (Some(x), Some(y)) if x == y
        );
        println!(
            "      {:?} == {:?}  → {}",
            m.port_map.get(&k_lhs),
            m.port_map.get(&k_rhs),
            res
        );
        res
    })
}

// ────────────────────────────────────────────────────
// object-safe Module trait
// ────────────────────────────────────────────────────
pub trait Module {
    fn file_path(&self)   -> PathBuf;
    fn module_name(&self) -> String;
    fn instance_name(&self) -> &str;

    fn submodules(&self)  -> Vec<&dyn Module> { Vec::new() }
    fn connections(&self) -> &HashSet<Connection> { &EMPTY_CONNECTIONS }

    // public façade – starts recursion with empty prefix
    fn query_net(&self, addr: &str)
        -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError>
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

// convenience: same helper without making the trait non-object-safe
pub fn run_svql_query_leaf<A: ToSocketAddrs>(
    addr: A,
    file: PathBuf,
    module: String,
) -> Result<Vec<SanitizedQueryMatch>, SvqlQueryError> {
    let mut cfg = SvqlRuntimeConfig::default();
    cfg.pat_filename    = file.to_string_lossy().into_owned();
    cfg.pat_module_name = module;
    cfg.verbose         = true;
    run_svql_query(addr, &cfg)
}