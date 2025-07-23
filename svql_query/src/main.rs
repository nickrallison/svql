use crate::driver::Driver;
use crate::module::RtlModule;
use examples::and::And;

mod driver;
mod examples;
mod module;
mod ports;
mod query;

fn main() {
    let driver = Driver::new_net("127.0.0.1:9999".to_string());

    let and1 = And::new("and1".to_string());

    // object-safe call
    let res = and1.query(&driver).unwrap();
    // let res = and1.query_net("127.0.0.1:9999").unwrap();
    for m in &res {
        println!("{:#?}", m);
    }
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
