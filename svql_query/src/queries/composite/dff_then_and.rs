use crate::impl_find_port;
use crate::instance::Instance;
use crate::queries::netlist::basic::and::and_gate::AndGate;
use crate::queries::netlist::basic::dff::Sdffe;
use crate::{State, WithPath};

/// DFF -> AND composite:
/// - Require that sdffe.q drives either and.a or and.b (one must hold).
#[derive(Debug, Clone)]
pub struct DffThenAnd<S>
where
    S: State,
{
    pub path: Instance,
    pub sdffe: Sdffe<S>,
    pub andg: AndGate<S>,
}

impl<S> WithPath<S> for DffThenAnd<S>
where
    S: State,
{
    impl_find_port!(DffThenAnd, sdffe, andg);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> DffThenAnd<S>
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

#[cfg(test)]
mod tests {
    use svql_driver::{Driver, DriverKey};
    use svql_subgraph::Config;

    use crate::{
        Search,
        instance::Instance,
        queries::{composite::dff_then_and::DffThenAnd, netlist::basic::dff::Sdffe},
    };

    #[test]
    fn test_dff_then_and() {
        let driver = Driver::new_workspace();

        let context = Sdffe::context(driver);

        // haystack
        let haystack_path = "examples/fixtures/basic/ff/verilog/and_q_double_sdffe.v";
        let haystack_module_name = "and_q_double_sdffe";
        let haystack = DriverKey::new(haystack_path.to_string(), haystack_module_name.to_string());

        // root path for the composite
        let root = Instance::root("dff_then_and".to_string());
        let config = Config::builder().exact_length().none().build();

        // run composite query
        // fn query<'ctx>(
        //     haystack_key: &DriverKey,
        //     context: &'ctx Context,
        //     path: Instance,
        //     config: &Config,
        // ) -> Vec<Self::Hit<'ctx, 'ctx>>;

        let hits = DffThenAnd::<Search>::query(&haystack, &context, root, &config);

        trace!("main: DffThenAnd matches={}", hits.len());

        assert_eq!(hits.len(), 2, "expected 2 DffThenAnd matches");

        for (k, h) in hits.iter().enumerate() {
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

            trace!(
                "hit[{}]: q={:?} a_src={:?} b_src={:?}",
                k,
                q.debug_index(),
                a_src.debug_index(),
                b_src.debug_index()
            );

            assert!(
                q == a_src || q == b_src,
                "expected sdffe.q to drive either andg.a or andg.b"
            );

            println!("hit[{}]: {:#?}", k, h);
        }

        Ok(())
    }
}
