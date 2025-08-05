// ########################
// Examples
// ########################

use itertools::iproduct;
use crate::instance::Instance;
use crate::{lookup, Connection, Match, QueryMatch, Search, State, Wire, WithPath};
use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::driver::Driver;
use crate::netlist::{Netlist, SearchableNetlist};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And<S> where S: State {
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S> WithPath<S> for And<S>
where S: State {
    fn new(path: Instance) -> Self {
        let a = Wire::new(path.child("a".to_string()));
        let b = Wire::new(path.child("b".to_string()));
        let y = Wire::new(path.child("y".to_string()));
        Self { path, a, b, y }
    }

    crate::impl_find_port!(And, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Netlist<S> for And<S>
where S: State {
    const MODULE_NAME        : &'static str = "and_gate";
    const FILE_PATH          : &'static str = "./examples/patterns/basic/and/verilog/and.v";
    const YOSYS              : &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN : &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for And<Search> {
    type Hit = And<Match>;

    fn from_query_match(m: QueryMatch, path:Instance) -> Self::Hit {
        let a = Match { id: lookup(&m.port_map,"a").cloned().unwrap() };
        let b = Match { id: lookup(&m.port_map,"b").cloned().unwrap() };
        let y = Match { id: lookup(&m.port_map,"y").cloned().unwrap() };
        And::<Match> {
            path: path.clone(),
            a: Wire::with_val(path.child("a".into()), a),
            b: Wire::with_val(path.child("b".into()), b),
            y: Wire::with_val(path.child("y".into()), y),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleAnd<S> where S: State {
    pub path: Instance,
    pub and1: And<S>,
    pub and2: And<S>,
}

impl<S> WithPath<S> for DoubleAnd<S>
where S: State {
    fn new(path: Instance) -> Self {
        let and1 = And::new(path.child("and1".to_string()));
        let and2 = And::new(path.child("and2".to_string()));
        Self { path, and1, and2 }
    }

    crate::impl_find_port!(DoubleAnd, and1, and2);
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for DoubleAnd<S>
where S: State {
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections: Vec<Vec<Connection<S>>> = Vec::new();
        let connection1 = Connection {
            from: self.and1.y.clone(),
            to: self.and2.a.clone(),
        };
        let connection2 = Connection {
            from: self.and1.y.clone(),
            to: self.and2.b.clone(),
        };
        let mut set = Vec::new();
        set.push(connection1);
        set.push(connection2);
        connections.push(set);
        connections
    }

}

impl SearchableComposite for DoubleAnd<Search> {
    type Hit = DoubleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        let and1_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and1".to_string()));
        let and2_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and2".to_string()));
        let results = iproduct!(and1_search_result, and2_search_result)
            .map(|(and1, and2)| {
                Self::Hit { and1, and2, path: path.clone() }
            })
            .filter(|s| {
                Self::Hit::validate_connections(s, s.connections())
            })
            .collect::<Vec<_>>();
        results
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripleAnd<S> where S: State {
    pub path: Instance,
    pub double_and: DoubleAnd<S>,
    pub and: And<S>,
}

impl <S> WithPath<S> for TripleAnd<S>
where S: State {
    fn new(path: Instance) -> Self {
        let double_and = DoubleAnd::new(path.child("double_and".to_string()));
        let and = And::new(path.child("and".to_string()));
        Self { path, double_and, and }
    }

    crate::impl_find_port!(TripleAnd, double_and, and);
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for TripleAnd<S>
where S: State {
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections: Vec<Vec<Connection<S>>> = Vec::new();
        let connection1 = Connection {
            from: self.double_and.and2.y.clone(),
            to: self.and.a.clone(),
        };
        let connection2 = Connection {
            from: self.double_and.and2.y.clone(),
            to: self.and.b.clone(),
        };
        let mut set = Vec::new();
        set.push(connection1);
        set.push(connection2);
        connections.push(set);
        connections
    }
}

impl SearchableComposite for TripleAnd<Search> {
    type Hit = TripleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        let double_and_search_result: Vec<DoubleAnd<Match>> = DoubleAnd::<Search>::query(driver, path.child("double_and".to_string()));
        let and_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and".to_string()));
        let results = iproduct!(double_and_search_result, and_search_result)
            .map(|(double_and, and)| {
                Self::Hit { double_and, and, path: path.clone() }
            })
            .filter(|s| {
                Self::Hit::validate_connections(s, s.connections())
            })
            .collect::<Vec<_>>();
        results
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherTripleAnd<S> where S: State {
    pub path: Instance,
    pub and1: And<S>,
    pub and2: And<S>,
    pub and3: And<S>,
}

impl<S> WithPath<S> for OtherTripleAnd<S>
where S: State {
    fn new(path: Instance) -> Self {
        let and1 = And::new(path.child("and1".to_string()));
        let and2 = And::new(path.child("and2".to_string()));
        let and3 = And::new(path.child("and3".to_string()));
        Self { path, and1, and2, and3 }
    }

    crate::impl_find_port!(OtherTripleAnd, and1, and2, and3);
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for OtherTripleAnd<S>
where S: State {

    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections: Vec<Vec<Connection<S>>> = Vec::new();

        let connection1 = Connection {
            from: self.and1.y.clone(),
            to: self.and2.a.clone(),
        };
        let connection2 = Connection {
            from: self.and1.y.clone(),
            to: self.and2.b.clone(),
        };
        let mut set = Vec::new();
        set.push(connection1);
        set.push(connection2);
        connections.push(set);

        let connection1 = Connection {
            from: self.and2.y.clone(),
            to: self.and3.a.clone(),
        };

        let connection2 = Connection {
            from: self.and2.y.clone(),
            to: self.and3.b.clone(),
        };

        let mut set = Vec::new();
        set.push(connection1);
        set.push(connection2);
        connections.push(set);

        connections
    }
}

impl SearchableComposite for OtherTripleAnd<Search> {
    type Hit = OtherTripleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        let and1_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and1".to_string()));
        let and2_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and2".to_string()));
        let and3_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and3".to_string()));
        let results = iproduct!(and1_search_result, and2_search_result, and3_search_result)
            .map(|(and1, and2, and3)| {
                Self::Hit { and1, and2, and3, path: path.clone() }
            })
            .filter(|s| {
                Self::Hit::validate_connections(s, s.connections())
            })
            .collect::<Vec<_>>();
        results
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveAnd<S> where S: State {
    pub path: Instance,
    pub and: And<S>,
    pub rec_and: Option<Box<RecursiveAnd<S>>>,
}

impl<S> RecursiveAnd<S>
where S: State {
    pub fn size(&self) -> usize {
        let mut size = 1; // Count this instance
        if let Some(recursive) = &self.rec_and {
            size += recursive.size()
        }
        size
    }
}

impl<S> WithPath<S> for RecursiveAnd<S>
where S: State {
    fn new(path: Instance) -> Self {
        let and = And::new(path.child("and".to_string()));
        let rec_and = None;
        Self { path, and, rec_and }
    }
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_str()) {
            Some("and") => self.and.find_port(p),
            Some("rec_and") => {
                if let Some(recursive) = &self.rec_and {
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

impl<S> Composite<S> for RecursiveAnd<S>
where S: State {

    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections = Vec::new();
        if let Some(recursive) = &self.rec_and {
            let connection1 = Connection {
                from: self.and.y.clone(),
                to: recursive.and.a.clone(),
            };
            let connection2 = Connection {
                from: self.and.y.clone(),
                to: recursive.and.b.clone(),
            };
            let mut set = Vec::new();
            set.push(connection1);
            set.push(connection2);
            connections.push(set);
        }
        connections

    }
}

impl SearchableComposite for RecursiveAnd<Search> {
    type Hit = RecursiveAnd<Match>;

    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        fn chain_to_recursive(
            chain: &[And<Match>],
            path: &Instance,
        ) -> RecursiveAnd<Match> {
            let head_and = chain[0].clone();

            if chain.len() == 1 {
                RecursiveAnd {
                    path: path.clone(),
                    and: head_and,
                    rec_and: None,
                }
            } else {
                let inner_path = path.child("rec_and".to_string());
                let tail = chain_to_recursive(&chain[1..], &inner_path);
                RecursiveAnd {
                    path: path.clone(),
                    and: head_and,
                    rec_and: Some(Box::new(tail)),
                }
            }
        }

        fn build_chains(
            driver: &Driver,
            cur_path: &Instance,
            first_and: &And<Match>,
        ) -> Vec<Vec<And<Match>>> {
            let mut chains: Vec<Vec<And<Match>>> = vec![vec![first_and.clone()]];
            let next_block_path = cur_path.child("rec_and".to_string());
            let next_and_path   = next_block_path.child("and".to_string());
            let inner_ands: Vec<And<Match>> = And::<Search>::query(driver, next_and_path);
            let this_y_id = first_and
                .y
                .val
                .as_ref()
                .map(|m| &m.id);

            for inner in inner_ands {
                let inner_a_id = inner
                    .a
                    .val
                    .as_ref()
                    .map(|m| &m.id);

                if this_y_id == inner_a_id {
                    let tails = build_chains(driver, &next_block_path, &inner);
                    for mut tail in tails {
                        tail.insert(0, first_and.clone());
                        chains.push(tail);
                    }
                }
            }

            chains
        }

        let and_hits: Vec<And<Match>> = And::<Search>::query(driver, path.child("and".to_string()));
        if and_hits.is_empty() {
            return Vec::new();
        }

        let mut all_hits: Vec<RecursiveAnd<Match>> = Vec::new();
        for top_and in &and_hits {
            let chains = build_chains(driver, &path, top_and);

            for chain in chains {
                let rec_hit = chain_to_recursive(&chain, &path);
                if rec_hit.validate_connections(rec_hit.connections()) {
                    all_hits.push(rec_hit);
                }
            }
        }

        let mut uniq_hits: Vec<RecursiveAnd<Match>> = Vec::new();
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
    fn test_and_netlist() {
        let driver = Driver::new_mock();

        let and = And::<Search>::root("and".to_string());
        assert_eq!(and.path().inst_path(), "and");
        assert_eq!(and.a.path.inst_path(), "and.a");
        assert_eq!(and.b.path.inst_path(), "and.b");
        assert_eq!(and.y.path.inst_path(), "and.y");

        let and_search_result = And::<Search>::query(&driver, and.path());
        assert_eq!(and_search_result.len(), 3, "Expected 3 matches for And, got {}", and_search_result.len());
    }

    // ###############
    // Composite Tests
    // ###############
    #[test]
    fn test_double_and_composite() {
        let driver = Driver::new_mock();
        let double_and = DoubleAnd::<Search>::root("double_and".to_string());
        assert_eq!(double_and.path().inst_path(), "double_and");
        assert_eq!(double_and.and1.path().inst_path(), "double_and.and1");
        assert_eq!(double_and.and2.path().inst_path(), "double_and.and2");
        assert_eq!(double_and.and1.y.path.inst_path(), "double_and.and1.y");
        let double_and_search_result = DoubleAnd::<Search>::query(&driver, double_and.path());
        assert_eq!(double_and_search_result.len(), 2, "Expected 2 matches for DoubleAnd, got {}", double_and_search_result.len());
    }

    #[test]
    fn test_triple_and_composite() {
        let driver = Driver::new_mock();
        let triple_and = TripleAnd::<Search>::root("triple_and");
        assert_eq!(triple_and.path().inst_path(), "triple_and");
        assert_eq!(triple_and.double_and.path().inst_path(), "triple_and.double_and");
        assert_eq!(triple_and.and.y.path.inst_path(), "triple_and.and.y");
        let triple_and_search_result = TripleAnd::<Search>::query(&driver, triple_and.path());
        assert_eq!(triple_and_search_result.len(), 1, "Expected 1 match for TripleAnd, got {}", triple_and_search_result.len());
    }

    #[test]
    fn test_other_triple_and_composite() {
        let driver = Driver::new_mock();
        let other_triple_and = OtherTripleAnd::<Search>::root("other_triple_and");
        assert_eq!(other_triple_and.path().inst_path(), "other_triple_and");
        assert_eq!(other_triple_and.and1.path().inst_path(), "other_triple_and.and1");
        assert_eq!(other_triple_and.and2.path().inst_path(), "other_triple_and.and2");
        assert_eq!(other_triple_and.and3.path().inst_path(), "other_triple_and.and3");
        let other_triple_and_search_result = OtherTripleAnd::<Search>::query(&driver, other_triple_and.path());
        assert_eq!(other_triple_and_search_result.len(), 1, "Expected 1 match for OtherTripleAnd, got {}", other_triple_and_search_result.len());
    }

    #[test]
    fn test_recursive_and_composite() {
        let driver = Driver::new_mock();
        let rec_and = RecursiveAnd::<Search>::root("rec_and");
        assert_eq!(rec_and.path().inst_path(), "rec_and");
        assert_eq!(rec_and.and.path().inst_path(), "rec_and.and");
        assert_eq!(rec_and.rec_and.is_none(), true, "Expected rec_and.rec_and to be None, got {:?}", rec_and.rec_and);
        let rec_and_search_result = RecursiveAnd::<Search>::query(&driver, rec_and.path());
        assert_eq!(rec_and_search_result.len(), 6, "Expected 6 matches for RecursiveAnd, got {}", rec_and_search_result.len());
    }

}