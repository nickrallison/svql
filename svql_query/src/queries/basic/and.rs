// ########################
// Examples
// ########################

use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::driver::Driver;
use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{lookup, Connection, Match, QueryMatch, Search, State, Wire, WithPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

impl<S> WithPath<S> for And<S>
where
    S: State,
{
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
where
    S: State,
{
    const MODULE_NAME: &'static str = "and_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/and/verilog/and.v";
    const YOSYS: &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN: &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for And<Search> {
    type Hit = And<Match>;

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
        And::<Match> {
            path: path.clone(),
            a: Wire::with_val(path.child("a".into()), a),
            b: Wire::with_val(path.child("b".into()), b),
            y: Wire::with_val(path.child("y".into()), y),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub and: And<S>,
    pub rec_and_1: Option<Box<RecursiveAnd<S>>>,
    pub rec_and_2: Option<Box<RecursiveAnd<S>>>,
}

impl<S> RecursiveAnd<S>
where
    S: State,
{
    pub fn size(&self) -> usize {
        let mut size = 1; // Count this instance
        if let Some(recursive) = &self.rec_and_1 {
            size += recursive.size()
        }
        if let Some(recursive) = &self.rec_and_2 {
            size += recursive.size()
        }
        size
    }
}

impl<S> WithPath<S> for RecursiveAnd<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let and = And::new(path.child("and".to_string()));
        let rec_and_1 = None;
        let rec_and_2 = None;
        Self {
            path,
            and,
            rec_and_1,
            rec_and_2,
        }
    }
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_str()) {
            Some("and") => self.and.find_port(p),
            Some("rec_and_1") => {
                if let Some(recursive) = &self.rec_and_1 {
                    recursive.find_port(p)
                } else {
                    None
                }
            }
            Some("rec_and_2") => {
                if let Some(recursive) = &self.rec_and_2 {
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
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections = Vec::new();
        if let Some(recursive) = &self.rec_and_1 {
            let connection1 = Connection {
                from: self.and.a.clone(),
                to: recursive.and.y.clone(),
            };
            let connection2 = Connection {
                from: self.and.b.clone(),
                to: recursive.and.y.clone(),
            };
            let mut set = Vec::new();
            set.push(connection1);
            set.push(connection2);
            connections.push(set);
        }
        if let Some(recursive) = &self.rec_and_2 {
            let connection1 = Connection {
                from: self.and.a.clone(),
                to: recursive.and.y.clone(),
            };
            let connection2 = Connection {
                from: self.and.b.clone(),
                to: recursive.and.y.clone(),
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
        todo!()
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
        assert_eq!(
            and_search_result.len(),
            3,
            "Expected 3 matches for And, got {}",
            and_search_result.len()
        );
    }

    // ###############
    // Composite Tests
    // ###############

    #[test]
    fn test_recursive_and_composite() {
        let driver = Driver::new_mock();
        let rec_and = RecursiveAnd::<Search>::root("rec_and");
        assert_eq!(rec_and.path().inst_path(), "rec_and");
        assert_eq!(rec_and.and.path().inst_path(), "rec_and.and");
        assert_eq!(
            rec_and.rec_and_1.is_none(),
            true,
            "Expected rec_and.rec_and_1 to be None, got {:?}",
            rec_and.rec_and_1
        );
        let rec_and_search_result = RecursiveAnd::<Search>::query(&driver, rec_and.path());
        assert_eq!(
            rec_and_search_result.len(),
            6,
            "Expected 6 matches for RecursiveAnd, got {}",
            rec_and_search_result.len()
        );
    }
}
