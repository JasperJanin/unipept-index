use std::process::exit;
use std::ptr::copy;
use fm_index::BackwardSearchIndex;
use fm_index_benchmarking::load_easy_fm_index;
// use fm_index::search;
// use fm_index::search::Search;
use fm_index::search::Search;
use fm_index::FMIndex;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampledArray;
use std::str;
use fm_searcher::{BDFMSearch, BidirectionalIndex};

// MNRTVKVAILGSGNIGTDLMYKILKKRWVLELSMIAGIDPQSEGLARARAEGVYATAGGIDAILEDPEIKIVFDATSAKAHLKHAKRLKEAGKVAIDLTPAAVGPYVVPPVNLMEHVDKDNVNLITCGGQATIPLVYAVSRVANVKYAEMVSTVSSSSAGPGTRQNIDEFTFTTSRGLEVIGGAEKGKAIIILNPAKPPILMRNTVYIAYEDGDDHQIRHSIGQMIHDVQQYVPGYRLKGEPIFDRRETPKGRLDVVILLLEVEGAGDFLPVSAGNLDIMTASAKQVGEVIAKRLIEMTSTA
struct SearchState<'a> {
    search: BDFMSearch<'a>,
    errors: u8,
    chars_taken: usize,
}

fn extend_search(state: &mut Vec<SearchState>, part: String, permitted_mistakes: i32, forward: bool) {
    
}

fn searchscheme_test() -> Vec<u64> {
    let alphabet = "ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    

    fn string_to_internal(index_rep: &str) -> String {
        index_rep
            .chars()
            .map(|c| match c {
                '-' => '@',
                _ => c.clone(),
            })
            .collect()
    }
    let index = BidirectionalIndex::load();
    
    let searchtermparts = vec!["AB", "BC"];
    let permitted_mistakes = vec![1, 0];
    
    let mut results = Vec::new();
    
    let mut statestack = Vec::new();
    statestack.push(SearchState { search: index.search(searchtermparts[1].to_string(), false), errors: 0, chars_taken: searchtermparts[1].len() });
    
    // 
    // while statestack.len() > 0 {
    //     let state = statestack.pop().unwrap();
    //     
    //     // check for total result
    //     if state.chars_taken >= searchterm.len() {
    //         results.append(&mut state.search.locate());
    //         continue;
    //     }
    //     
    //     let part = 2 - state.chars_taken / 9;
    //     let next_char_a = [searchterm[searchterm.len() - 1 - state.chars_taken]];
    //     let next_char = str::from_utf8(&next_char_a).unwrap();
    // 
    //     // errors allowed
    //     if state.errors < permitted_mistakes[part] {
    //         for c in alphabet {
    //             let search = Search {
    //                 index: &index,
    //                 s: state.search.s.clone(),
    //                 e: state.search.e.clone(),
    //                 pattern: state.search.pattern.clone()
    //             };
    //             search.search_backward(next_char);
    //             
    //             if search.count() > 0 {
    //                 // match/mismatch
    //                 statestack.push(SearchState {
    //                     search: Search {
    //                         index: &search.index,
    //                         s: search.s.clone(),
    //                         e: search.e.clone(),
    //                         pattern: search.pattern.clone(),
    //                     },
    //                     errors: if c.eq(&searchterm[searchterm.len() - 1 - state.chars_taken]) { state.errors } else { state.errors + 1 },
    //                     chars_taken: state.chars_taken + 1
    //                 });
    //                 
    //                 // insertion
    //                 statestack.push(SearchState {
    //                     search,
    //                     errors: state.errors + 1,
    //                     chars_taken: state.chars_taken
    //                 });
    //         }}
    //         
    //         // deletion
    //         let search = state.search.search_backward(next_char);
    //         statestack.push(SearchState {
    //             search,
    //             errors: state.errors + 1,
    //             chars_taken: state.chars_taken + 1
    //         })
    //     }
    //         
    //     // errors not allowed
    //     else {
    //         let search = state.search.search_backward(next_char);
    //         if search.count() > 0 {
    //             statestack.push(SearchState{
    //                 search,
    //                 errors: state.errors,
    //                 chars_taken: state.chars_taken + 1
    //             });
    //             
    //         }
    //    }
    // 
    // }
    results
}

fn main() {
    let results = searchscheme_test();
    println!("RESULTS: {:?}\n\nCount = {}", results, results.len());
}
