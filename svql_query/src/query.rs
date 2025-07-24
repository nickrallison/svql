use std::collections::{hash_map::Entry, HashSet};
use svql_common::mat::{IdString, SanitizedQueryMatch};

use crate::ports::{Connection, InPort, OutPort};

use lazy_static::lazy_static;

use log::trace;

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
    trace!("try_merge_matches()");
    trace!("m1.port_map = {:#?}", m1.port_map);
    trace!("m2.port_map = {:#?}", m2.port_map);

    let mut port_map = m1.port_map.clone();
    for (k, v) in &m2.port_map {
        match port_map.entry(k.clone()) {
            Entry::Vacant(e) => {
                e.insert(v.clone());
            }
            Entry::Occupied(e) => {
                if e.get() != v {
                    trace!("Port maps conflict on key {:?}: {:?} != {:?}", k, e.get(), v);
                    return None;
                }
            }
        }
    }

    let mut cell_map = m1.cell_map.clone();
    for (k, v) in &m2.cell_map {
        match cell_map.entry(k.clone()) {
            Entry::Vacant(e) => {
                e.insert(v.clone());
            }
            Entry::Occupied(e) => {
                if e.get() != v {
                    trace!("Cell maps conflict on key {:?}: {:?} != {:?}", k, e.get(), v);
                    return None;
                }
            }
        }
    }

    trace!("merged OK");
    trace!("port_map = {:#?}", port_map);
    trace!("cell_map = {:#?}", cell_map);
    Some(SanitizedQueryMatch { port_map, cell_map })
}

pub fn connections_satisfied(
    m: &SanitizedQueryMatch,
    conns: &HashSet<Connection<InPort, OutPort>>,
) -> bool {
    trace!("connections_satisfied()");
    trace!("match port_map: {:#?}", m.port_map);
    
    for c in conns {
        trace!("checking connection: {} == {}", c.in_port.0, c.out_port.0);
    }

    let result = conns.iter().all(|c| {
        let k_lhs = IdString::Named(c.in_port.0.clone());
        let k_rhs = IdString::Named(c.out_port.0.clone());

        let lhs_value = m.port_map.get(&k_lhs);
        let rhs_value = m.port_map.get(&k_rhs);
        
        trace!("Looking for keys: {:?} and {:?}", k_lhs, k_rhs);
        trace!("Values found: {:?} and {:?}", lhs_value, rhs_value);

        let res = matches!(
            (lhs_value, rhs_value),
            (Some(x), Some(y)) if x == y
        );
        
        trace!(
            "{:?} == {:?}  → {}",
            lhs_value,
            rhs_value,
            res
        );
        res
    });
    
    trace!("connections_satisfied result: {}", result);
    result
}
