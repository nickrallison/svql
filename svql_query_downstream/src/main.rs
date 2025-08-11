use svql_query::composite::SearchableComposite;
use svql_driver_handler::driver::mock::and_three::MockDriverThreeAnd;
use svql_driver_handler::driver::Driver;
use svql_query::queries::basic::and::RecursiveAnd;
use svql_query::{Match, Search, WithPath};

use crate::and::AndAB;

mod and;

fn main() {
    // let mock_and = MockDriverThreeAnd::new();
    let driver = Driver::new_net("localhost:9999".to_string());

    let and_ab = AndAB::<Search>::root("rec_and");
    let and_ab_search_result: Vec<AndAB<Match>> =
        AndAB::<Search>::query(&driver, and_ab.path());
    assert_eq!(
        and_ab_search_result.len(),
        6,
        "Expected 6 matches for AndAB, got {}",
        and_ab_search_result.len()
    );
    // for match_ in and_ab_search_result {
    //     println!("Match Size: {}", match_.size());
    //     // println!("Match: {:#?}", match_);
    // }
}
