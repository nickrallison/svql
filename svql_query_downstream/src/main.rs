use svql_query::composite::SearchableComposite;
use svql_query::driver::mock::MockDriverThreeAnd;
use svql_query::driver::Driver;
use svql_query::queries::basic::and::RecursiveAnd;
use svql_query::{Match, Search, WithPath};

mod and;

fn main() {
    let mock_and = MockDriverThreeAnd::new();
    let driver = Driver::new_mock(mock_and.into());

    let rec_and = RecursiveAnd::<Search>::root("rec_and");
    let rec_and_search_result: Vec<RecursiveAnd<Match>> =
        RecursiveAnd::<Search>::query(&driver, rec_and.path());
    assert_eq!(
        rec_and_search_result.len(),
        6,
        "Expected 6 matches for RecursiveAnd, got {}",
        rec_and_search_result.len()
    );
    for match_ in rec_and_search_result {
        println!("Match Size: {}", match_.size());
        // println!("Match: {:#?}", match_);
    }
}
