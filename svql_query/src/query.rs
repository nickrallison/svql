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
    net::{run_svql_query, SvqlQueryError},
};
use crate::ports::{Connection, InPort, OutPort};

lazy_static! {
    static ref EMPTY_CONNECTIONS: HashSet<Connection<InPort, OutPort>> = HashSet::new();
}

// ────────────────────────────────────────────────────
// helpers (unchanged except for the pub fields)
// ────────────────────────────────────────────────────
pub fn try_merge_matches(
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

pub fn connections_satisfied(m: &SanitizedQueryMatch, conns: &HashSet<Connection<InPort, OutPort>>,) -> bool {

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