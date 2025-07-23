use crate::driver::Driver;
use crate::examples::triple_and::{TripleAnd, TripleAndResult};
use crate::module::{RtlModuleTrait, RtlQuery};

mod driver;
mod examples;
mod module;
mod ports;
mod query;

fn main() {
    let driver = Driver::new_net("127.0.0.1:9999".to_string());

    let triple_and: RtlQuery<TripleAnd> = RtlQuery::new("triple_and".to_string(), TripleAnd::new());

    let res = triple_and.query::<TripleAndResult>(&driver).unwrap();
    for m in &res {
        println!("{:#?}", m);
    }
    println!("---");
}
