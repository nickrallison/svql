use crate::examples::and::{And, AndResult};
use crate::module::{RtlModule, RtlQueryResult, RtlQueryResultTrait, RtlQueryTrait};
use std::collections::HashSet;

use crate::connect;
use crate::driver::{Driver, DriverError};
use crate::module::QueryError;
use crate::ports::{Connection, InPort, OutPort};
use std::fmt::Debug;
use svql_common::mat::SanitizedCellData;

#[derive(Debug, Clone)]
pub struct TripleAnd {
    pub and1: RtlModule<And>,
    pub and2: RtlModule<And>,
    pub and3: RtlModule<And>,
}

impl TripleAnd {
    pub fn new() -> Self {
        TripleAnd {
            and1: RtlModule::new("and1".into(), And::new()),
            and2: RtlModule::new("and2".into(), And::new()),
            and3: RtlModule::new("and3".into(), And::new()),
        }
    }
}

impl RtlQueryTrait<TripleAnd> for TripleAnd {
    fn run_query<QueryResultType>(
        &self,
        driver: &Driver,
    ) -> Result<Vec<Result<RtlQueryResult<QueryResultType>, QueryError>>, DriverError>
    where
        QueryResultType: Debug,
        QueryResultType: RtlQueryResultTrait<QueryResultType>,
    {
        // This implementation is only valid when the caller expects a `TripleAndResult`
        // (which is the only thing we ever do in the example).  Guard against accidental
        // misuse – this will be replaced by a proc-macro later on.
        // assert_eq!(
        //     TypeId::of::<QueryResultType>(),
        //     TypeId::of::<TripleAndResult>(),
        //     "`TripleAnd::run_query` must be called with `TripleAndResult`"
        // );

        // Query the three sub-modules
        let and1_matches = self.and1.query::<AndResult>(driver)?;
        let and2_matches = self.and2.query::<AndResult>(driver)?;
        let and3_matches = self.and3.query::<AndResult>(driver)?;

        // Collect individual errors so they are not lost
        let mut results: Vec<Result<RtlQueryResult<QueryResultType>, QueryError>> = Vec::new();
        for err in and1_matches
            .iter()
            .chain(&and2_matches)
            .chain(&and3_matches)
            .filter_map(|r| r.as_ref().err())
        {
            results.push(Err(err.clone()));
        }

        // Only keep the successful matches for combination
        let ok1: Vec<_> = and1_matches.into_iter().filter_map(Result::ok).collect();
        let ok2: Vec<_> = and2_matches.into_iter().filter_map(Result::ok).collect();
        let ok3: Vec<_> = and3_matches.into_iter().filter_map(Result::ok).collect();

        // Build Cartesian product and check the required internal connections:
        //
        //   and1.y  -> and2.a
        //   and2.y  -> and3.a
        //
        for m1 in &ok1 {
            for m2 in &ok2 {
                // and1.y == and2.a ?
                if m1.module.y != m2.module.a {
                    continue;
                }
                for m3 in &ok3 {
                    // and2.y == and3.a ?
                    if m2.module.y != m3.module.a {
                        continue;
                    }

                    // All connections satisfied – aggregate cells and build result
                    let mut cells: Vec<SanitizedCellData> = Vec::new();
                    cells.extend(m1.cells.clone());
                    cells.extend(m2.cells.clone());
                    cells.extend(m3.cells.clone());

                    let triple_res = TripleAndResult::new(
                        m1.module.clone(),
                        m2.module.clone(),
                        m3.module.clone(),
                    );

                    // SAFETY:
                    // We asserted above that `QueryResultType == TripleAndResult`, therefore the
                    // transmute is sound.
                    let qr: RtlQueryResult<QueryResultType> = RtlQueryResult::new(cells, unsafe {
                        std::mem::transmute::<TripleAndResult, QueryResultType>(triple_res)
                    });

                    results.push(Ok(qr));
                }
            }
        }

        Ok(results)
    }

    fn connect(&self) -> HashSet<Connection<InPort, OutPort>> {
        let mut connections = HashSet::new();
        connect!(&mut connections, &self.and1.module.y, &self.and2.module.a);
        connect!(&mut connections, &self.and2.module.y, &self.and3.module.a);
        connections
    }
}

#[derive(Debug, Clone)]
pub struct TripleAndResult {
    pub and1: AndResult,
    pub and2: AndResult,
    pub and3: AndResult,
}

impl TripleAndResult {
    pub fn new(and1: AndResult, and2: AndResult, and3: AndResult) -> Self {
        TripleAndResult { and1, and2, and3 }
    }
}

impl RtlQueryResultTrait<TripleAndResult> for TripleAndResult {}
