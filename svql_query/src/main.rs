use crate::driver::Driver;
use crate::examples::and::And;
use crate::examples::triple_and::TripleAnd;
use crate::module::{RtlModule, RtlModuleTrait, RtlQuery};

mod driver;
mod examples;
mod module;
mod ports;
mod query;

fn main() {
    let driver = Driver::new_net("127.0.0.1:9999".to_string());

    let and: RtlModule<And> = RtlModule::new("and".to_string(), And::new());
    let res = and.query(&driver).unwrap();
    for m in &res {
        println!("{:#?}", m);
    }

    println!("---");

    let triple_and: RtlQuery<TripleAnd> = RtlQuery::new("triple_and".to_string(), TripleAnd::new());

    let res = triple_and.query(&driver).unwrap();
    for m in &res {
        println!("{:#?}", m);
    }
    println!("---");
}
