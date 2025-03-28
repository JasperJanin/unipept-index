use fm_searcher::bd_index::BidirectionalIndex;

fn main() {
    let index = BidirectionalIndex::load();
    
    // MA-CAR-ON-
    // MASCARPONE
    let search_term = "MASCARPONE".to_string();

    let results = index.find_approximate_matches(search_term, 3);
    
    println!("RESULTS: {:?}\n\nCount = {}", results, results.len());
}
