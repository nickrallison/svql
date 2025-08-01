use svql_query::*;
use svql_query::driver::Driver;

fn main() {

    let driver = Driver::new_mock();

    let rec_and = RecursiveAnd::<Search>::root("rec_and");
    let rec_and_search_result: Vec<RecursiveAnd<Match>> = RecursiveAnd::<Search>::query(&driver, rec_and.path());
    assert_eq!(rec_and_search_result.len(), 6, "Expected 6 matches for RecursiveAnd, got {}", rec_and_search_result.len());
    for match_ in rec_and_search_result {
        println!("Match Size: {}", match_.size());
        // println!("Match: {:#?}", match_);
        
    }
}
