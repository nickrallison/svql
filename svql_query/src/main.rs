use std::{net::ToSocketAddrs, path::PathBuf};
use std::collections::HashSet;
use svql_common::mat::{IdString, SanitizedCellData, SanitizedQueryMatch};
use crate::{
    ports::{InPort, OutPort, Connection},
    query::{run_svql_query_leaf},
};
use crate::module::Module;
use crate::net::SvqlQueryError;

mod query;
mod ports;
mod net;
mod module;


#[derive(Debug)]
struct And<In, Out, Inst, Id> {
    inst: Inst,
    connections: HashSet<Connection<In, Out>>,
    // 
    a: In,
    b: In,
    y: Out,
    id: Id,
}

impl Module for And<InPort, OutPort, String, ()> {
    type Result = And<IdString, IdString, (), HashSet<SanitizedCellData>>;

    fn file_path(&self)   -> PathBuf { "svql_query/verilog/and.v".into() }
    fn module_name(&self) -> String  { "and_gate".into() }
    fn instance_name(&self) -> &str  { &self.inst }

    fn query_net(
        &self,
        addr: &str,
    ) -> Result<Vec<Self::Result>, SvqlQueryError> {
        // identical to today’s implementation
        let res: Vec<SanitizedQueryMatch> = run_svql_query_leaf(addr, self.file_path(), self.module_name())?;
        let results: Vec<Self::Result> = res.into_iter()
            .map(|m| {
                // And<IdString, IdString, (), HashSet<SanitizedCellData>>

                let and_query_result: Self::Result = And {
                    inst: self.instance_name().to_string(),
                    connections: HashSet::new(),
                    a: m.port_map.get(&IdString::Named("a".into())).expect("Input port 'a' not found")
                        .clone(),
                    b: m.port_map.get(&IdString::Named("b".into())).expect("Output port 'b' not found")
                        .clone(),
                    y: m.port_map.get(&IdString::Named("y".into())).expect("Output port 'y' not found")
                        .clone(),
                    id: m.cell_map.into_iter().map(|(_, v)| {
                        v
                    }).collect(),
                };
                and_query_result
            })
            .collect();
        
    }
}

fn main() {
    let and1 = AndQuery::new("and1");

    // object-safe call
    let res = and1.query_net("127.0.0.1:9999").unwrap();
    for m in &res { println!("{}", m); }
    println!("---");

    // let and2 = AndQuery::new("and2");
    // let and3 = AndQuery::new("and");

    // let mut sub_combined = SubCombinedAnd::new("SubCombinedAnd", and1, and2);
    // let mut combined = CombinedAnd::new("CombinedAnd", and3, sub_combined).connect();

    // println!("CombinedAnd: {:#?}", combined);

    // let res2 = combined.query_net("127.0.0.1:9999").unwrap();

    // println!("---");
    // println!("CombinedAnd query result:");
    // for m in &res2 { println!("{}", m); }
}