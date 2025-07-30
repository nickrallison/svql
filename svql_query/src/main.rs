use crate::instance::Instance;
use std::collections::HashSet;

mod instance;

// ########################
// Base Search Types
// ########################

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Search;
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Match;

// ########################
// Driver
// ########################

pub struct Driver;

// ########################
// Traits
// ########################

pub trait Searchable: Clone + Default {
    type Hit;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
    fn permutations(&self) -> Vec<Self>;
}

pub trait Netlist {
    fn module_name() -> &'static str;
    fn file_path() -> &'static str;
    fn yosys() -> &'static str;
    fn svql_driver_plugin() -> &'static str;

    // ####
    fn swappable() -> Vec<HashSet<String>>;
}

pub trait Composite {}

// ########################
// Containers
// ########################

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wire<T> {
    pub path: Instance,
    pub val: T,
}

impl <T: Default> Wire<T> {
    pub fn root(name: String) -> Self {
        let path = Instance::root(name);
        Self::new(path)
    }
    pub fn new(path: Instance) -> Self {
        Self { path, val: T::default() }
    }
}

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

impl Searchable for And<Search> {
    type Hit = And<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("This should be a simple call to the driver's query method");
    }
}

impl<T> Netlist for And<T> {
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
    fn swappable() -> Vec<HashSet<String>> {
        vec![HashSet::from(["a".to_string(), "b".to_string()])]
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

impl Searchable for DoubleAnd<Search> {
    type Hit = DoubleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("This should be calls to query for each And in DoubleAnd, compose them with itertools' cartesian product, then a filter to combine them");
    }
}

impl<T> Composite for DoubleAnd<T> {}

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

impl Searchable for TripleAnd<Search> {
    type Hit = TripleAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("This should look similar to DoubleAnd's query, but with a call to double_and, and then a call to and");
    }
}

pub enum RecursiveAnd<T> {
    BaseCase(And<T>),
    RecursiveCase(Box<RecursiveAnd<T>>),
}

impl<T> RecursiveAnd<T> {
    pub fn root_base(name: String) -> Self {
        let path = Instance::root(name);
        Self::new_base(path, base)
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

impl Searchable for RecursiveAnd<Search> {
    type Hit = RecursiveAnd<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        todo!("TODO: Work upwards from the base case and filtering each step until no more is found");
    }
}

fn main() {
    let and_search: And<Search> = And::root("and".to_string());
    let double_and_search: DoubleAnd<Search> = DoubleAnd::root("double_and".to_string());
    let triple_and_search: TripleAnd<Search> = TripleAnd::root("triple_and".to_string());



    // can't define enum, don't need to specify which type, just want all of them

    // let and_matches: Vec<And<Match>> = and_search.query().unwrap();
    // let double_and_matches: Vec<DoubleAnd<Match>> = double_and_search.query().unwrap();
    // let triple_and_matches: Vec<TripleAnd<Match>> = triple_and_search.query().unwrap();
    //... 
}