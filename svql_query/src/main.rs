    use crate::{driver::Driver, instance::Instance};
    use std::{collections::{HashMap, HashSet}, hash::Hash, sync::Arc, vec};
    use itertools::{iproduct};
    use svql_common::{config::ffi::SvqlRuntimeConfig, id_string::IdString};

    mod instance;
    mod driver;

    // ########################
    // Base Search Types
    // ########################

    #[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
    pub struct Search;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Match {
        pub id: IdString
    }

    impl Default for Match {
        fn default() -> Self {
            Self { id: IdString::Named("default".into()) }
        }
    }

    // ########################
    // Helpers
    // ########################


    pub fn lookup<'a>(m: &'a HashMap<IdString, IdString>, pin: &str) -> Option<&'a IdString> {
        m.get(&IdString::Named(pin.into()))
    }

    // ########################
    // Traits
    // ########################

    type QueryMatch = svql_common::matches::SanitizedQueryMatch;

    pub trait Netlist<T> {

        type Tuple;

        fn into_tuple(self) -> Self::Tuple;
        fn from_tuple(tuple: Self::Tuple) -> Self;

        // ####

        fn module_name() -> &'static str;
        fn file_path() -> &'static str;
        fn yosys() -> &'static str;
        fn svql_driver_plugin() -> &'static str;

        // ####
        fn config() -> SvqlRuntimeConfig {
            let mut cfg = SvqlRuntimeConfig::default();
            cfg.pat_filename = Self::file_path().to_string();
            cfg.pat_module_name = Self::module_name().to_string();
            cfg.verbose = true;
            cfg
        }
        fn path(&self) -> Instance;
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>>;
    }

    pub trait SearchableNetlist: Netlist<Search> {
        type Hit;
        fn from_query_match(match_: QueryMatch, path: Instance) -> Self::Hit;
        fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
    }

    pub trait Composite<T> {
        type Tuple;
        
        fn into_tuple(self) -> Self::Tuple;
        fn from_tuple(tuple: Self::Tuple) -> Self;

        fn connections(&self) -> Vec<Vec<Connection<T>>>;
        fn path(&self) -> Instance;
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>>;
    }

    pub trait SearchableComposite: Composite<Search> {
        type Hit;
        fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
    }

    pub trait MatchedComposite: Composite<Match> {
        fn validate_connection(&self, connection: Connection<Match>) -> bool;
        fn validate_connections(&self, connections: Vec<Vec<Connection<Match>>>) -> bool;
    }

    impl<T> MatchedComposite for T where T: Composite<Match> {
        fn validate_connection(&self, connection: Connection<Match>) -> bool {
            let in_port_id = self.find_port(&connection.from.path);
            let out_port_id = self.find_port(&connection.to.path);

            if let (Some(in_port), Some(out_port)) = (in_port_id, out_port_id) {
                return in_port.val == out_port.val;
            }
            false
        }
        fn validate_connections(&self, connections: Vec<Vec<Connection<Match>>>) -> bool {
            for connection_set in connections {
                // each set needs to contain at least one valid connection
                let mut valid = false;
                for conn in connection_set {
                    if self.validate_connection(conn) {
                        valid = true;
                        break;
                    }
                }
                if !valid {
                    return false;
                }
            }
            true
        }
    }

    // ########################
    // Containers
    // ########################

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Wire<T> {
        pub path: Instance,
        pub val: Option<T>,
    }

    impl <T: Default> Wire<T> {
        pub fn root(name: String) -> Self {
            let path = Instance::root(name);
            Self::new(path)
        }
        pub fn new(path: Instance) -> Self {
            Self { path, val: None }
        }
        pub fn with_val(path: Instance, val: T) -> Self {
            Self { path, val: Some(val) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Connection<T> {
        pub from: Wire<T>,
        pub to: Wire<T>,
    }

    // ########################
    // Examples
    // ########################

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct And<T> {
        pub path: Instance,
        pub a: Wire<T>,
        pub b: Wire<T>,
        pub y: Wire<T>,
    }

    impl<T: Default> And<T> {
        pub fn root(name: String) -> Self {
            let path = Instance::root(name);
            Self::new(path)
        }
        pub fn new(path: Instance) -> Self {
            let a = Wire::new(path.child("a".to_string()));
            let b = Wire::new(path.child("b".to_string()));
            let y = Wire::new(path.child("y".to_string()));
            Self { path, a, b, y }
        }
    }

    impl SearchableNetlist for And<Search> {
        type Hit = And<Match>;
        fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
            let results = driver.query(&Self::config())
                .expect("Failed to query driver")
                .map(|match_| {
                    Self::from_query_match(match_, path.clone())
                })
                .collect();
            results
        }
            
        fn from_query_match(match_: QueryMatch, path: Instance) -> Self::Hit {
            let a: Match = Match { id: lookup(&match_.port_map, "a").cloned().expect(concat!("Port a not found")) };
            let b: Match = Match { id: lookup(&match_.port_map, "b").cloned().expect(concat!("Port b not found")) };
            let y: Match = Match { id: lookup(&match_.port_map, "y").cloned().expect(concat!("Port y not found")) };

            Self::Hit {
                a: Wire::with_val(path.child("a".to_string()), a.clone()),
                b: Wire::with_val(path.child("b".to_string()), b.clone()),
                y: Wire::with_val(path.child("y".to_string()), y.clone()),
                path: path.clone(),
            }
        }
    }

    impl<T> Netlist<T> for And<T> {

        type Tuple = (Wire<T>, Wire<T>, Wire<T>, Instance);

        fn into_tuple(self) -> Self::Tuple {
            (self.a, self.b, self.y, self.path)
        }
        
        fn from_tuple(tuple: Self::Tuple) -> Self {
            let (a, b, y, path) = tuple;
            Self { a, b, y, path }
        }

        fn module_name() -> &'static str {
            "and_gate"
        }
        fn file_path() -> &'static str {
            "./examples/patterns/basic/and/verilog/and.v"
        }
        fn yosys() -> &'static str {
            "./yosys/yosys"
        }
        fn svql_driver_plugin() -> &'static str {
            "./build/svql_driver/libsvql_driver.so"
        }

        // ##################
        fn path(&self) -> Instance {
            self.path.clone()
        }
        
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>> {
            let self_height = self.path.height();
            let child_height = self_height + 1;
            let child_name = port_name.get_item(child_height);
            if let Some(name) = child_name {
                if name == Arc::new("a".to_string()) {
                    return Some(&self.a);
                } else if name == Arc::new("b".to_string()) {
                    return Some(&self.b);
                } else if name == Arc::new("y".to_string()) {
                    return Some(&self.y);
                }
            }
            None
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DoubleAnd<T> {
        pub path: Instance,
        pub and1: And<T>,
        pub and2: And<T>,
    }

    impl<T: Default> DoubleAnd<T> {
        pub fn root(name: String) -> Self {
            let path = Instance::root(name);
            Self::new(path)
        }
        pub fn new(path: Instance) -> Self {
            let and1 = And::new(path.child("and1".to_string()));
            let and2 = And::new(path.child("and2".to_string()));
            Self { path, and1, and2 }
        }
    }

    impl<T> Composite<T> for DoubleAnd<T> where 
        T: Clone + Eq + Hash {
        type Tuple = (And<T>, And<T>, Instance);
        fn into_tuple(self) -> Self::Tuple {
            (self.and1, self.and2, self.path)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            let (and1, and2, path) = tuple;
            Self { and1, and2, path }
        }
        
        fn connections(&self) -> Vec<Vec<Connection<T>>> {
            let mut connections: Vec<Vec<Connection<T>>> = Vec::new();
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
        fn path(&self) -> Instance {
            self.path.clone()
        }
        
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>> {
            let self_height = self.path.height();
            let child_height = self_height + 1;
            let child_name = port_name.get_item(child_height);
            if let Some(name) = child_name {
                if name == Arc::new("and1".to_string()) {
                    return self.and1.find_port(port_name);
                } else if name == Arc::new("and2".to_string()) {
                    return self.and2.find_port(port_name);
                }
            }
            None
        }

    }

    impl SearchableComposite for DoubleAnd<Search> {
        type Hit = DoubleAnd<Match>;
        fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
            let and1_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and1".to_string()));
            let and2_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and2".to_string()));
            let results = iproduct!(and1_search_result, and2_search_result)
                .map(|(and1, and2)| {
                    Self::Hit::from_tuple((and1, and2, path.clone()))
                })
                .filter(|s| {
                    Self::Hit::validate_connections(s, s.connections())
                })
                .collect::<Vec<_>>();
            results
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct TripleAnd<T> {
        pub path: Instance,
        pub double_and: DoubleAnd<T>,
        pub and: And<T>,
    }

    impl<T: Default> TripleAnd<T> {
        pub fn root(name: String) -> Self {
            let path = Instance::root(name);
            Self::new(path)
        }
        pub fn new(path: Instance) -> Self {
            let double_and = DoubleAnd::new(path.child("double_and".to_string()));
            let and = And::new(path.child("and".to_string()));
            Self { path, double_and, and }
        }
    }

    impl<T> Composite<T> for TripleAnd<T> where 
        T: Clone + Eq + Hash {
        type Tuple = (DoubleAnd<T>, And<T>, Instance);
        fn into_tuple(self) -> Self::Tuple {
            (self.double_and, self.and, self.path)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            let (double_and, and, path) = tuple;
            Self { double_and, and, path }
        }
        
        fn connections(&self) -> Vec<Vec<Connection<T>>> {
            let mut connections: Vec<Vec<Connection<T>>> = Vec::new();
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
        fn path(&self) -> Instance {
            self.path.clone()
        }
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>> {
            let self_height = self.path.height();
            let child_height = self_height + 1;
            let child_name = port_name.get_item(child_height);
            if let Some(name) = child_name {
                if name == Arc::new("double_and".to_string()) {
                    return self.double_and.find_port(port_name);
                } else if name == Arc::new("and".to_string()) {
                    return self.and.find_port(port_name);
                }
            }
            None
        }
    }

    impl SearchableComposite for TripleAnd<Search> {
        type Hit = TripleAnd<Match>;
        fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
            let double_and_search_result: Vec<DoubleAnd<Match>> = DoubleAnd::<Search>::query(driver, path.child("double_and".to_string()));
            let and_search_result: Vec<And<Match>> = And::<Search>::query(driver, path.child("and".to_string()));
            let results = iproduct!(double_and_search_result, and_search_result)
                .map(|(double_and, and)| {
                    Self::Hit::from_tuple((double_and, and, path.clone()))
                })
                .filter(|s| {
                    Self::Hit::validate_connections(s, s.connections())
                })
                .collect::<Vec<_>>();
            results
        }
    }


    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct OtherTripleAnd<T> {
        pub path: Instance,
        pub and1: And<T>,
        pub and2: And<T>,
        pub and3: And<T>,
    }

    impl<T: Default> OtherTripleAnd<T> {
        pub fn root(name: String) -> Self {
            let path = Instance::root(name);
            Self::new(path)
        }
        pub fn new(path: Instance) -> Self {
            let and1 = And::new(path.child("and1".to_string()));
            let and2 = And::new(path.child("and2".to_string()));
            let and3 = And::new(path.child("and3".to_string()));
            Self { path, and1, and2, and3 }
        }
    }

    impl<T> Composite<T> for OtherTripleAnd<T> where 
        T: Clone + Eq + Hash {
        type Tuple = (And<T>, And<T>, And<T>, Instance);
        fn into_tuple(self) -> Self::Tuple {
            (self.and1, self.and2, self.and3, self.path)
        }
        fn from_tuple(tuple: Self::Tuple) -> Self {
            let (and1, and2, and3, path) = tuple;
            Self { and1, and2, and3, path }
        }
        
        fn connections(&self) -> Vec<Vec<Connection<T>>> {
            let mut connections: Vec<Vec<Connection<T>>> = Vec::new();

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
        fn path(&self) -> Instance {
            self.path.clone()
        }
        
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>> {
            let self_height = self.path.height();
            let child_height = self_height + 1;
            let child_name = port_name.get_item(child_height);
            if let Some(name) = child_name {
                if name == Arc::new("and1".to_string()) {
                    return self.and1.find_port(port_name);
                } else if name == Arc::new("and2".to_string()) {
                    return self.and2.find_port(port_name);
                } else if name == Arc::new("and3".to_string()) {
                    return self.and3.find_port(port_name);
                }
            }
            None
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
                    Self::Hit::from_tuple((and1, and2, and3, path.clone()))
                })
                .filter(|s| {
                    Self::Hit::validate_connections(s, s.connections())
                })
                .collect::<Vec<_>>();
            results
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct RecursiveAnd<T> {
        pub path: Instance,
        pub and: And<T>,
        pub rec_and: Option<Box<RecursiveAnd<T>>>,
    }

    impl<T: Default> RecursiveAnd<T> {
        pub fn root_base(name: String) -> Self {
            let path = Instance::root(name);
            Self::new(path)
        }
        pub fn new(path: Instance) -> Self {
            let and = And::new(path.child("and".to_string()));
            let rec_and = None;
            Self { path, and, rec_and }
        }
    }

    impl<T> Composite<T> for RecursiveAnd<T> where
        T: Clone + Eq + Hash {
        type Tuple = (And<T>, Option<Box<RecursiveAnd<T>>>, Instance);

        fn into_tuple(self) -> Self::Tuple {
            (self.and, self.rec_and, self.path)
        }

        fn from_tuple(tuple: Self::Tuple) -> Self {
            let (and, rec_and, path) = tuple;
            Self { and, rec_and, path }
        }

        fn connections(&self) -> Vec<Vec<Connection<T>>> {
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

        fn path(&self) -> Instance {
            self.path.clone()
        }
        fn find_port(&self, port_name: &Instance) -> Option<&Wire<T>> {
            let self_height = self.path.height();
            let child_height = self_height + 1;
            let child_name = port_name.get_item(child_height);
            if let Some(name) = child_name {
                if name == Arc::new("and".to_string()) {
                    return self.and.find_port(port_name);
                }
                if name == Arc::new("rec_and".to_string()) {
                    if let Some(recursive) = &self.rec_and {
                        return recursive.find_port(port_name);
                    }
                }
            }
            None
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
    
    fn main() {

        let driver = Driver::new_mock();

        let rec_and = RecursiveAnd::<Search>::root_base("rec_and".into());
        let rec_and_search_result: Vec<RecursiveAnd<Match>> = RecursiveAnd::<Search>::query(&driver, rec_and.path());
        assert_eq!(rec_and_search_result.len(), 6, "Expected 6 matches for RecursiveAnd, got {}", rec_and_search_result.len());

    }

    #[cfg(test)] 
    mod tests {
        use std::sync::Arc;

        use super::*;

        // ###############
        // Instance Tests
        // ###############
        #[test]
        fn test_instance() {
            let inst = Instance::root("test".to_string());
            let child1 = inst.child("child1".to_string());
            let child2 = child1.child("child2".to_string());
            assert_eq!(inst.inst_path(), "test");
            assert_eq!(child1.inst_path(), "test.child1");
            assert_eq!(child2.inst_path(), "test.child1.child2");
            assert_eq!(child2.get_item(0), Some(Arc::new("test".to_string())));
            assert_eq!(child2.get_item(1), Some(Arc::new("child1".to_string())));
            assert_eq!(child2.get_item(2), Some(Arc::new("child2".to_string())));
            assert_eq!(child2.get_item(3), None);
        }

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
            let triple_and = TripleAnd::<Search>::root("triple_and".to_string());
            assert_eq!(triple_and.path().inst_path(), "triple_and");
            assert_eq!(triple_and.double_and.path().inst_path(), "triple_and.double_and");
            assert_eq!(triple_and.and.y.path.inst_path(), "triple_and.and.y");
            let triple_and_search_result = TripleAnd::<Search>::query(&driver, triple_and.path());
            assert_eq!(triple_and_search_result.len(), 1, "Expected 1 match for TripleAnd, got {}", triple_and_search_result.len());
        }

        #[test]
        fn test_other_triple_and_composite() {
            let driver = Driver::new_mock();
            let other_triple_and = OtherTripleAnd::<Search>::root("other_triple_and".to_string());
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
            let rec_and = RecursiveAnd::<Search>::root_base("rec_and".to_string());
            assert_eq!(rec_and.path().inst_path(), "rec_and");
            assert_eq!(rec_and.and.path().inst_path(), "rec_and.and");
            assert_eq!(rec_and.rec_and.is_none(), true, "Expected rec_and.rec_and to be None, got {:?}", rec_and.rec_and);
            let rec_and_search_result = RecursiveAnd::<Search>::query(&driver, rec_and.path());
            assert_eq!(rec_and_search_result.len(), 6, "Expected 6 matches for RecursiveAnd, got {}", rec_and_search_result.len());
        }

    }