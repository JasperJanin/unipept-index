use fm_searcher::approximate_search::searchscheme_test;
use fm_searcher::bd_index::BidirectionalIndex;

fn main() {
    let index = BidirectionalIndex::load();

    let search_term = "BCCDEEEDDCCDB".to_string();

    let results = index.find_approximate_matches(search_term, 2);
    
    println!("RESULTS: {:?}\n\nCount = {}", results, results.len());
}
