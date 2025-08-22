// svql_query/src/queries/composite/dff_then_and.rs
use svql_driver::{Context, Driver, DriverKey};
use svql_subgraph::Config;

use crate::{
    Connection, Match, Search, State, WithPath,
    composite::{Composite, MatchedComposite, SearchableComposite},
    instance::Instance,
    netlist::SearchableNetlist,
    queries::netlist::basic::{and::AndGate, dff::Sdffe},
};
use itertools::iproduct;

#[derive(Debug, Clone)]
pub struct SdffeThenAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub sdffe: Sdffe<S>,
    pub andg: AndGate<S>,
}

impl<S> SdffeThenAnd<S>
where
    S: State,
{
    pub fn new(path: Instance) -> Self {
        Self {
            path: path.clone(),
            sdffe: Sdffe::new(path.child("sdffe".to_string())),
            andg: AndGate::new(path.child("andg".to_string())),
        }
    }
}

impl<S> WithPath<S> for SdffeThenAnd<S>
where
    S: State,
{
    crate::impl_find_port!(SdffeThenAnd, sdffe, andg);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for SdffeThenAnd<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        // Define the connection: sdffe.q -> (andg.a OR andg.b)
        vec![vec![
            Connection {
                from: self.sdffe.q.clone(),
                to: self.andg.a.clone(),
            },
            Connection {
                from: self.sdffe.q.clone(),
                to: self.andg.b.clone(),
            },
        ]]
    }
}

impl<'ctx> MatchedComposite<'ctx> for SdffeThenAnd<Match<'ctx>> {}

impl SearchableComposite for SdffeThenAnd<Search> {
    type Hit<'ctx> = SdffeThenAnd<Match<'ctx>>;

    fn context(driver: &Driver) -> Result<Context, Box<dyn std::error::Error>> {
        let sdffe_context = Sdffe::<Search>::context(driver)?;
        let and_context = AndGate::<Search>::context(driver)?;
        Ok(sdffe_context.merge(and_context))
    }

    fn query<'ctx>(
        haystack_key: &DriverKey,
        context: &'ctx Context,
        path: Instance,
        config: &Config,
    ) -> Vec<Self::Hit<'ctx>> {
        // First get individual matches
        let sdffe_matches = Sdffe::<Search>::query(
            haystack_key,
            context,
            path.child("sdffe".to_string()),
            config,
        );
        let and_matches = AndGate::<Search>::query(
            haystack_key,
            context,
            path.child("andg".to_string()),
            config,
        );

        // Create composite instances

        iproduct!(sdffe_matches, and_matches)
            .map(|(sdffe, andg)| SdffeThenAnd {
                path: path.clone(),
                sdffe,
                andg,
            })
            .filter(|composite| composite.validate_connections(composite.connections()))
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use svql_driver::Driver;
    use svql_subgraph::Config;

    use crate::{
        Search, composite::SearchableComposite, instance::Instance,
        queries::composite::dff_then_and::SdffeThenAnd,
    };

    #[test]
    fn test_dff_then_and() {
        let driver = Driver::new_workspace().expect("Failed to create driver");

        let context =
            SdffeThenAnd::<Search>::context(&driver).expect("Failed to create context for Sdffe");

        // haystack
        let haystack_path = "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v";
        let haystack_module_name = "and_q_double_sdffe";
        let (haystack_key, haystack) = driver
            .get_or_load_design(haystack_path, haystack_module_name.to_string())
            .expect("Failed to get haystack design");

        // root path for the composite
        let root = Instance::root("dff_then_and".to_string());
        let config = Config::builder().exact_length().none().build();

        let context = context.with_design(haystack_key.clone(), haystack.clone());

        let hits = SdffeThenAnd::<Search>::query(&haystack_key, &context, root, &config);

        assert_eq!(hits.len(), 2, "expected 2 DffThenAnd matches");

        for (_, h) in hits.iter().enumerate() {
            let q = h
                .sdffe
                .q
                .val
                .as_ref()
                .expect("missing sdffe.q binding")
                .design_cell_ref
                .expect("sdffe.q should have a design driver");

            let a_src = h
                .andg
                .a
                .val
                .as_ref()
                .expect("missing andg.a binding")
                .design_cell_ref
                .expect("andg.a should have a design source");
            let b_src = h
                .andg
                .b
                .val
                .as_ref()
                .expect("missing andg.b binding")
                .design_cell_ref
                .expect("andg.b should have a design source");

            assert!(
                q == a_src || q == b_src,
                "expected sdffe.q to drive either andg.a or andg.b"
            );
        }
    }
}
