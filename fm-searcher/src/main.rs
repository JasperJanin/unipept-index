use fm_searcher::approximate_search::searchscheme_test;

fn main() {
    let results = searchscheme_test();
    println!("RESULTS: {:?}\n\nCount = {}", results, results.len());
}
