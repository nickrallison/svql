// ########################
// Examples
// ########################

use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::driver::Driver;
use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{lookup, Connection, Match, QueryMatch, Search, State, Wire, WithPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Or<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S> WithPath<S> for Or<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let a = Wire::new(path.child("a".to_string()));
        let b = Wire::new(path.child("b".to_string()));
        let y = Wire::new(path.child("y".to_string()));
        Self { path, a, b, y }
    }

    crate::impl_find_port!(Or, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for Or<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "or_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/or/verilog/or.v";
    const YOSYS: &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN: &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for Or<Search> {
    type Hit = Or<Match>;

    fn from_query_match(m: QueryMatch, path: Instance) -> Self::Hit {
        let a = Match {
            id: lookup(&m.port_map, "a").cloned().unwrap(),
        };
        let b = Match {
            id: lookup(&m.port_map, "b").cloned().unwrap(),
        };
        let y = Match {
            id: lookup(&m.port_map, "y").cloned().unwrap(),
        };
        Or::<Match> {
            path: path.clone(),
            a: Wire::with_val(path.child("a".into()), a),
            b: Wire::with_val(path.child("b".into()), b),
            y: Wire::with_val(path.child("y".into()), y),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveOr<S>
where
    S: State,
{
    pub path: Instance,
    pub or: Or<S>,
    pub rec_or: Option<Box<RecursiveOr<S>>>,
}

impl<S> RecursiveOr<S>
where
    S: State,
{
    pub fn size(&self) -> usize {
        let mut size = 1; // Count this instance
        if let Some(recursive) = &self.rec_or {
            size += recursive.size()
        }
        size
    }
}

impl<S> WithPath<S> for RecursiveOr<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let or = Or::new(path.child("or".to_string()));
        let rec_or = None;
        Self { path, or, rec_or }
    }
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_str()) {
            Some("or") => self.or.find_port(p),
            Some("rec_or") => {
                if let Some(recursive) = &self.rec_or {
                    recursive.find_port(p)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for RecursiveOr<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections = Vec::new();
        if let Some(recursive) = &self.rec_or {
            let connection1 = Connection {
                from: self.or.y.clone(),
                to: recursive.or.a.clone(),
            };
            let connection2 = Connection {
                from: self.or.y.clone(),
                to: recursive.or.b.clone(),
            };
            let mut set = Vec::new();
            set.push(connection1);
            set.push(connection2);
            connections.push(set);
        }
        connections
    }
}

impl SearchableComposite for RecursiveOr<Search> {
    type Hit = RecursiveOr<Match>;

    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        fn chain_to_recursive(chain: &[Or<Match>], path: &Instance) -> RecursiveOr<Match> {
            let head_or = chain[0].clone();

            if chain.len() == 1 {
                RecursiveOr {
                    path: path.clone(),
                    or: head_or,
                    rec_or: None,
                }
            } else {
                let inner_path = path.child("rec_or".to_string());
                let tail = chain_to_recursive(&chain[1..], &inner_path);
                RecursiveOr {
                    path: path.clone(),
                    or: head_or,
                    rec_or: Some(Box::new(tail)),
                }
            }
        }

        fn build_chains(
            driver: &Driver,
            cur_path: &Instance,
            first_or: &Or<Match>,
        ) -> Vec<Vec<Or<Match>>> {
            let mut chains: Vec<Vec<Or<Match>>> = vec![vec![first_or.clone()]];
            let next_block_path = cur_path.child("rec_or".to_string());
            let next_or_path = next_block_path.child("or".to_string());
            let inner_ors: Vec<Or<Match>> = Or::<Search>::query(driver, next_or_path);
            let this_y_id = first_or.y.val.as_ref().map(|m| &m.id);

            for inner in inner_ors {
                let inner_a_id = inner.a.val.as_ref().map(|m| &m.id);

                if this_y_id == inner_a_id {
                    let tails = build_chains(driver, &next_block_path, &inner);
                    for mut tail in tails {
                        tail.insert(0, first_or.clone());
                        chains.push(tail);
                    }
                }
            }

            chains
        }

        let or_hits: Vec<Or<Match>> = Or::<Search>::query(driver, path.child("or".to_string()));
        if or_hits.is_empty() {
            return Vec::new();
        }

        let mut all_hits: Vec<RecursiveOr<Match>> = Vec::new();
        for top_or in &or_hits {
            let chains = build_chains(driver, &path, top_or);

            for chain in chains {
                let rec_hit = chain_to_recursive(&chain, &path);
                if rec_hit.validate_connections(rec_hit.connections()) {
                    all_hits.push(rec_hit);
                }
            }
        }

        let mut uniq_hits: Vec<RecursiveOr<Match>> = Vec::new();
        for hit in all_hits {
            if !uniq_hits.contains(&hit) {
                uniq_hits.push(hit);
            }
        }

        uniq_hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ###############
    // Netlist Tests
    // ###############
    #[test]
    fn test_or_netlist() {
        let driver = Driver::new_mock();

        let or = Or::<Search>::root("or".to_string());
        assert_eq!(or.path().inst_path(), "or");
        assert_eq!(or.a.path.inst_path(), "or.a");
        assert_eq!(or.b.path.inst_path(), "or.b");
        assert_eq!(or.y.path.inst_path(), "or.y");

        let or_search_result = Or::<Search>::query(&driver, or.path());
        assert_eq!(
            or_search_result.len(),
            3,
            "Expected 3 matches for Or, got {}",
            or_search_result.len()
        );
    }

    // ###############
    // Composite Tests
    // ###############

    #[test]
    fn test_recursive_or_composite() {
        let driver = Driver::new_mock();
        let rec_or = RecursiveOr::<Search>::root("rec_or");
        assert_eq!(rec_or.path().inst_path(), "rec_or");
        assert_eq!(rec_or.or.path().inst_path(), "rec_or.or");
        assert_eq!(
            rec_or.rec_or.is_none(),
            true,
            "Expected rec_or.rec_or to be None, got {:?}",
            rec_or.rec_or
        );
        let rec_or_search_result = RecursiveOr::<Search>::query(&driver, rec_or.path());
        assert_eq!(
            rec_or_search_result.len(),
            6,
            "Expected 6 matches for RecursiveOr, got {}",
            rec_or_search_result.len()
        );
    }
}
