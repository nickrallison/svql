// ########################
// Examples
// ########################

use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::driver::Driver;
use crate::instance::Instance;
use crate::netlist::SearchableNetlist;
use crate::queries::basic::and::And;
use crate::{Connection, Match, Search, State, Wire, WithPath};
use itertools::iproduct;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoubleAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub and1: And<S>,
    pub and2: And<S>,
}

impl<S> WithPath<S> for DoubleAnd<S>
where
    S: State,
{
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
where
    S: State,
{
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
        let and1_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and1".to_string()));
        let and2_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and2".to_string()));
        // let other_filters: Vec<Box<dyn Fn(&Self) -> bool>> = Self::Hit::other_filters();
        let results = iproduct!(and1_search_result, and2_search_result)
            .map(|(and1, and2)| Self::Hit {
                and1,
                and2,
                path: path.clone(),
            })
            .filter(|s| {
                let conn_ok = s.validate_connections(s.connections());
                let other_ok = s.other_filters().iter().all(|f| f(s));
                conn_ok && other_ok
            })
            .collect::<Vec<_>>();
        
        results
    }
}

impl MatchedComposite for DoubleAnd<Match> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TripleAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub double_and: DoubleAnd<S>,
    pub and: And<S>,
}

impl<S> WithPath<S> for TripleAnd<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let double_and = DoubleAnd::new(path.child("double_and".to_string()));
        let and = And::new(path.child("and".to_string()));
        Self {
            path,
            double_and,
            and,
        }
    }

    crate::impl_find_port!(TripleAnd, double_and, and);
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for TripleAnd<S>
where
    S: State,
{
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
        let double_and_search_result: Vec<DoubleAnd<Match>> =
            DoubleAnd::<Search>::query(driver, path.child("double_and".to_string()));
        let and_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and".to_string()));
        let results = iproduct!(double_and_search_result, and_search_result)
            .map(|(double_and, and)| Self::Hit {
                double_and,
                and,
                path: path.clone(),
            })
            .filter(|s| {
                let conn_ok = s.validate_connections(s.connections());
                let other_ok = s.other_filters().iter().all(|f| f(s));
                conn_ok && other_ok
            })
            .collect::<Vec<_>>();
        results
    }
}

impl MatchedComposite for TripleAnd<Match> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherTripleAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub and1: And<S>,
    pub and2: And<S>,
    pub and3: And<S>,
}

impl<S> WithPath<S> for OtherTripleAnd<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let and1 = And::new(path.child("and1".to_string()));
        let and2 = And::new(path.child("and2".to_string()));
        let and3 = And::new(path.child("and3".to_string()));
        Self {
            path,
            and1,
            and2,
            and3,
        }
    }

    crate::impl_find_port!(OtherTripleAnd, and1, and2, and3);
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for OtherTripleAnd<S>
where
    S: State,
{
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
        let and1_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and1".to_string()));
        let and2_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and2".to_string()));
        let and3_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and3".to_string()));
        let results = iproduct!(and1_search_result, and2_search_result, and3_search_result)
            .map(|(and1, and2, and3)| Self::Hit {
                and1,
                and2,
                and3,
                path: path.clone(),
            })
            .filter(|s| {
                let conn_ok = s.validate_connections(s.connections());
                let other_ok = s.other_filters().iter().all(|f| f(s));
                conn_ok && other_ok
            })
            .collect::<Vec<_>>();
        results
    }
}

impl MatchedComposite for OtherTripleAnd<Match> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        vec![]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::mock::MockDriverThreeAnd;

    // ###############
    // Composite Tests
    // ###############
    #[test]
    fn test_double_and_composite() {
        let and_mock = MockDriverThreeAnd::new();
        let driver = Driver::new_mock(and_mock.into());

        let double_and = DoubleAnd::<Search>::root("double_and".to_string());
        assert_eq!(double_and.path().inst_path(), "double_and");
        assert_eq!(double_and.and1.path().inst_path(), "double_and.and1");
        assert_eq!(double_and.and2.path().inst_path(), "double_and.and2");
        assert_eq!(double_and.and1.y.path.inst_path(), "double_and.and1.y");
        let double_and_search_result = DoubleAnd::<Search>::query(&driver, double_and.path());
        assert_eq!(
            double_and_search_result.len(),
            2,
            "Expected 2 matches for DoubleAnd, got {}",
            double_and_search_result.len()
        );
    }

    #[test]
    fn test_triple_and_composite() {
        let and_mock = MockDriverThreeAnd::new();
        let driver = Driver::new_mock(and_mock.into());
        let triple_and = TripleAnd::<Search>::root("triple_and");
        assert_eq!(triple_and.path().inst_path(), "triple_and");
        assert_eq!(
            triple_and.double_and.path().inst_path(),
            "triple_and.double_and"
        );
        assert_eq!(triple_and.and.y.path.inst_path(), "triple_and.and.y");
        let triple_and_search_result = TripleAnd::<Search>::query(&driver, triple_and.path());
        assert_eq!(
            triple_and_search_result.len(),
            1,
            "Expected 1 match for TripleAnd, got {}",
            triple_and_search_result.len()
        );
    }

    #[test]
    fn test_other_triple_and_composite() {
        let and_mock = MockDriverThreeAnd::new();
        let driver = Driver::new_mock(and_mock.into());
        let other_triple_and = OtherTripleAnd::<Search>::root("other_triple_and");
        assert_eq!(other_triple_and.path().inst_path(), "other_triple_and");
        assert_eq!(
            other_triple_and.and1.path().inst_path(),
            "other_triple_and.and1"
        );
        assert_eq!(
            other_triple_and.and2.path().inst_path(),
            "other_triple_and.and2"
        );
        assert_eq!(
            other_triple_and.and3.path().inst_path(),
            "other_triple_and.and3"
        );
        let other_triple_and_search_result =
            OtherTripleAnd::<Search>::query(&driver, other_triple_and.path());
        assert_eq!(
            other_triple_and_search_result.len(),
            1,
            "Expected 1 match for OtherTripleAnd, got {}",
            other_triple_and_search_result.len()
        );
    }
}
