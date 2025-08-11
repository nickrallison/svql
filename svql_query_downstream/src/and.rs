use svql_common::id_string::IdString;
use svql_query::{
    composite::{Composite, MatchedComposite, SearchableComposite},
    driver::Driver,
    impl_find_port,
    instance::Instance,
    netlist::SearchableNetlist,
    queries::basic::and::And,
    Connection,
    Match,
    Search,
    State,
    WithPath,
};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref A_RE: Regex = Regex::new(r"^a$").unwrap();
    static ref B_RE: Regex = Regex::new(r"^b$").unwrap();
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndAB<S>
where
    S: State,
{
    pub path: Instance,
    pub and: And<S>,
}

impl<S> WithPath<S> for AndAB<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let and = And::new(path.child("and".to_string()));
        Self { path, and }
    }

    impl_find_port!(AndAB, and);
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for AndAB<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        vec![]
    }
}

impl SearchableComposite for AndAB<Search> {
    type Hit = AndAB<Match>;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        let and_search_result: Vec<And<Match>> =
            And::<Search>::query(driver, path.child("and".to_string()));
        let results = and_search_result
            .into_iter()
            .map(|and| Self::Hit {
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

impl MatchedComposite for AndAB<Match> {
    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
        let a_lambda = |s: &Self| {
            if let Some(port) = s.find_port(&s.and.a.path) {
                if let Some(Match { id: IdString::Named(name) }) = &port.val {
                    let a_is_match = A_RE.is_match(&name);
                    let b_is_match = B_RE.is_match(&name);
                    return a_is_match || b_is_match;
                }
            }
            false
        };
        let b_lambda = |s: &Self| {
            if let Some(port) = s.find_port(&s.and.b.path) {
                if let Some(Match { id: IdString::Named(name) }) = &port.val {
                    let a_is_match = A_RE.is_match(&name);
                    let b_is_match = B_RE.is_match(&name);
                    return a_is_match || b_is_match;
                }
            }
            false
        };
        vec![Box::new(a_lambda), Box::new(b_lambda)]
    }
}
