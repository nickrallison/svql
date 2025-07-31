use crate::{driver::Driver, instance::Instance};
use std::collections::{HashMap};
use itertools::{iproduct};
use svql_common::{config::ffi::SvqlRuntimeConfig, id_string::IdString};

mod instance;
mod driver;

// ########################
// Base Search Types
// ########################

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Search;
#[derive(Debug, Clone, PartialEq, Eq)]
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

// pub trait Searchable: Clone {
//     type Hit;
//     fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
// }

type QueryMatch = svql_common::matches::SanitizedQueryMatch;

pub trait Netlist {

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
}

pub trait SearchableNetlist: Netlist {
    type Hit;
    fn from_query_match(match_: QueryMatch, path: Instance) -> Self::Hit;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
}

pub trait Composite<T> {
    type Tuple;

    fn into_tuple(self) -> Self::Tuple;
    fn from_tuple(tuple: Self::Tuple) -> Self;

    fn connections(&self) -> Vec<Connection<T>>;
    fn path(&self) -> Instance;
}

pub trait SearchableComposite: Clone {
    type Hit;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
}

// ########################
// Permutation Traits
// ########################

pub trait PermutableNetlist where Self: Sized {
    fn permutations(self) -> Vec<Self>;
}

pub trait PermutableComposite: Sized {
    fn permutations(self) -> Vec<Self>;
}

// impl<T: Composite + Clone> PermutableComposite for T {
//     fn permutations(self) -> Vec<Self> {
//         todo!("Implement permutations over each component of the composite, then combine them");
//     }
// }

// ########################
// Containers
// ########################

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
            a: Wire::with_val(path.child("a".to_string()), a),
            b: Wire::with_val(path.child("b".to_string()), b),
            y: Wire::with_val(path.child("y".to_string()), y),
            path,
        }
    }
}

impl<T> Netlist for And<T> {

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
}

impl<T: Clone> PermutableNetlist for And<T>{
    fn permutations(self) -> Vec<Self> {

        let self_owned: And<T> = self;

        let a = self_owned.a.clone();
        let b = self_owned.b.clone();
        let y = self_owned.y.clone();

        let results = vec![
            Self::from_tuple((a.clone(), b.clone(), y.clone(), self_owned.path.clone())),
            Self::from_tuple((b.clone(), a.clone(), y.clone(), self_owned.path.clone())),
        ];
        results
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

impl SearchableComposite for DoubleAnd<Search> {
    type Hit = DoubleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("This should be calls to query for each And in DoubleAnd, compose them with itertools' cartesian product, then a filter to combine them");
    }
}

impl<T: Clone> Composite<T> for DoubleAnd<T> {
    type Tuple = (And<T>, And<T>, Instance);
    fn into_tuple(self) -> Self::Tuple {
        (self.and1, self.and2, self.path)
    }
    fn from_tuple(tuple: Self::Tuple) -> Self {
        let (and1, and2, path) = tuple;
        Self { and1, and2, path }
    }
    
    fn connections(&self) -> Vec<Connection<T>> {
        let mut connections = Vec::new();
        let mut connection = Connection {
            from: self.and1.a.clone(),
            to: self.and2.y.clone(),
        };
        connections.push(connection);
        connections
    }
    fn path(&self) -> Instance {
        self.path.clone()
    }

}

impl<T: Clone> PermutableComposite for DoubleAnd<T> {
    fn permutations(self) -> Vec<Self> {
        let and1_perms = PermutableNetlist::permutations(self.and1);
        let and2_perms = PermutableNetlist::permutations(self.and2);
        let results = iproduct!(and1_perms, and2_perms)
            .map(|(and1, and2)| Self::from_tuple((and1, and2, self.path.clone())))
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

impl<T> Composite<T> for TripleAnd<T> {
    type Tuple = (DoubleAnd<T>, And<T>, Instance);
    fn into_tuple(self) -> Self::Tuple {
        (self.double_and, self.and, self.path)
    }
    fn from_tuple(tuple: Self::Tuple) -> Self {
        let (double_and, and, path) = tuple;
        Self { double_and, and, path }
    }
    
    fn connections(&self) -> Vec<Connection<T>> {
        todo!()
    }
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl SearchableComposite for TripleAnd<Search> {
    type Hit = TripleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("This should look similar to DoubleAnd's query, but with a call to double_and, and then a call to and");
    }
}

impl<T: Clone> PermutableComposite for TripleAnd<T> {
    fn permutations(self) -> Vec<Self> {
        let double_and_perms = PermutableComposite::permutations(self.double_and);
        let and_perms = PermutableNetlist::permutations(self.and);
        
        let results = iproduct!(double_and_perms, and_perms)
            .map(|(double_and, and)| Self::from_tuple((double_and, and, self.path.clone())))
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

impl<T> Composite<T> for OtherTripleAnd<T> {
    type Tuple = (And<T>, And<T>, And<T>, Instance);
    fn into_tuple(self) -> Self::Tuple {
        (self.and1, self.and2, self.and3, self.path)
    }
    fn from_tuple(tuple: Self::Tuple) -> Self {
        let (and1, and2, and3, path) = tuple;
        Self { and1, and2, and3, path }
    }
    
    fn connections(&self) -> Vec<Connection<T>> {
        todo!()
    }
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl SearchableComposite for OtherTripleAnd<Search> {
    type Hit = OtherTripleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("This should look similar to DoubleAnd's query, but with a call to double_and, and then a call to and");
    }
}

impl<T: Clone> PermutableComposite for OtherTripleAnd<T> {
    fn permutations(self) -> Vec<Self> {
        let and1_perms = PermutableNetlist::permutations(self.and1);
        let and2_perms = PermutableNetlist::permutations(self.and2);
        let and3_perms = PermutableNetlist::permutations(self.and3);

        let results = iproduct!(and1_perms, and2_perms, and3_perms)
            .map(|(and1, and2, and3)| Self::from_tuple((and1, and2, and3, self.path.clone())))
            .collect::<Vec<_>>();

        results
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecursiveAnd<T> {
    BaseCase(And<T>),
    RecursiveCase(Box<RecursiveAnd<T>>),
}

impl<T: Default> RecursiveAnd<T> {
    pub fn root_base(name: String) -> Self {
        let path = Instance::root(name);
        Self::new_base(path)
    }
    pub fn new_base(path: Instance) -> Self {
        let and = And::new(path.child("and".to_string()));
        Self::BaseCase(and)
    }

    // ###

    pub fn recursive(val: RecursiveAnd<T>) -> Self {
        Self::RecursiveCase(Box::new(val))
    }
}

// impl<T> Composite for RecursiveAnd<T> {
//     type Tuple = (And<T>, Instance);
//     fn into_tuple(self) -> Self::Tuple {
//         match self {
//             RecursiveAnd::BaseCase(and) => (and, and.path.clone()),
//             RecursiveAnd::RecursiveCase(recursive) => {
//                 let (and, path) = recursive.into_tuple();
//                 (and, path)
//             }
//         }
//     }
//     fn from_tuple(tuple: Self::Tuple) -> Self {
//         let (and, path) = tuple;
//         Self::BaseCase(and)
//     }
// }

impl SearchableComposite for RecursiveAnd<Search> {
    type Hit = RecursiveAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("TODO: Work upwards from the base case and filtering each step until no more is found");
    }
}

fn main() {
    // let and_search: And<Search> = And::root("and".to_string());
    // let double_and_search: DoubleAnd<Search> = DoubleAnd::root("double_and".to_string());
    // let triple_and_search: TripleAnd<Search> = TripleAnd::root("triple_and".to_string());

    let driver = Driver::new_mock();

    let and: And<Search> = And::<Search>::root("and".into());
    let and_search_result: Vec<And<Match>> = And::<Search>::query(&driver, and.path());
    assert_eq!(and_search_result.len(), 3, "Expected 3 matches for And, got {}", and_search_result.len());

    let a_perms = PermutableNetlist::permutations(and);
    assert!(a_perms.len() == 2, "Expected 2 permutations for And, got {}", a_perms.len());

    let d_and = DoubleAnd::<Search>::root("d".into());
    let d_and_search_result: Vec<DoubleAnd<Match>> = DoubleAnd::<Search>::query(&driver, d_and.path());
    assert_eq!(d_and_search_result.len(), 2, "Expected 2 matches for DoubleAnd, got {}", d_and_search_result.len());

    let d_perms = PermutableComposite::permutations(d_and);
    assert!(d_perms.len() == 4, "Expected 4 permutations for DoubleAnd, got {}", d_perms.len());

    let t_and = TripleAnd::<Search>::root("t".into());
    let t_and_search_result: Vec<TripleAnd<Match>> = TripleAnd::<Search>::query(&driver, t_and.path());
    assert_eq!(t_and_search_result.len(), 1, "Expected 1 match for TripleAnd, got {}", t_and_search_result.len());

    let t_perms = PermutableComposite::permutations(t_and); // == 8 variants
    assert!(t_perms.len() == 8, "Expected 8 permutations for TripleAnd, got {}", t_perms.len());

    // OtherTripleAnd has (And permutations) × (And permutations) × (And permutations)
    let o_and = OtherTripleAnd::<Search>::root("o".into());
    let o_and_search_result: Vec<OtherTripleAnd<Match>> = OtherTripleAnd::<Search>::query(&driver, o_and.path());
    assert_eq!(o_and_search_result.len(), 1, "Expected 1 match for OtherTripleAnd, got {}", o_and_search_result.len());

    let o_perms = PermutableComposite::permutations(o_and); // == 8 variants
    assert!(o_perms.len() == 8, "Expected 8 permutations for OtherTripleAnd, got {}", o_perms.len());
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
}