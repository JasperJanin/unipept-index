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

// MNRTVKVAILGSGNIGTDLMYKILKKRWVLELSMIAGIDPQSEGLARARAEGVYATAGGIDAILEDPEIKIVFDATSAKAHLKHAKRLKEAGKVAIDLTPAAVGPYVVPPVNLMEHVDKDNVNLITCGGQATIPLVYAVSRVANVKYAEMVSTVSSSSAGPGTRQNIDEFTFTTSRGLEVIGGAEKGKAIIILNPAKPPILMRNTVYIAYEDGDDHQIRHSIGQMIHDVQQYVPGYRLKGEPIFDRRETPKGRLDVVILLLEVEGAGDFLPVSAGNLDIMTASAKQVGEVIAKRLIEMTSTA
// zoekterm: VAILGSGNIGTDLMYKILKKRWVLELSMZAGIDPQSEGLARARAEGVYATAGGIDAILEDPEIKIVFDATSAKAH
fn searchscheme_test() -> Vec<u64> {
    let alphabet = "ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    struct SearchState<'a> {
        search: Search<'a, FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>>,
        errors: u8,
        chars_taken: usize,
    }

    fn string_to_internal(index_rep: &str) -> String {
        index_rep
            .chars()
            .map(|c| match c {
                '-' => '@',
                _ => c.clone(),
            })
            .collect()
    }
    let index = load_easy_fm_index();
    let searchterm = "ATAGGIDAILEDPAIKIVFDATSAKAH".as_bytes();
    let searchtermparts = vec!["ATAGGIDAI", "LEDPAIKIV", "FDATSAKAH"];
    let permitted_mistakes = vec![2, 2, 0];

    let mut results = Vec::new();

    let mut statestack = Vec::new();
    statestack.push(SearchState { search: index.search_backward(searchtermparts[2]), errors: 0, chars_taken: searchtermparts[2].len() });

    while statestack.len() > 0 {
        let state = statestack.pop().unwrap();
        
        // check for total result
        if state.chars_taken >= searchterm.len()-5 {
            results.append(&mut state.search.locate());
            continue;
        }
        
        let part = 2 - state.chars_taken / 9;
        let next_char_a = [searchterm[searchterm.len() - 1 - state.chars_taken]];
        let next_char = str::from_utf8(&next_char_a).unwrap();

        // errors allowed
        if state.errors < permitted_mistakes[part] {
            for c in alphabet {
                let search = Search {
                    index: &index,
                    s: state.search.s.clone(),
                    e: state.search.e.clone(),
                    pattern: state.search.pattern.clone()
                };
                search.search_backward(next_char);
                
                if search.count() > 0 {
                    // match/mismatch
                    statestack.push(SearchState {
                        search: Search {
                            index: &search.index,
                            s: search.s.clone(),
                            e: search.e.clone(),
                            pattern: search.pattern.clone(),
                        },
                        errors: if c.eq(&searchterm[searchterm.len() - 1 - state.chars_taken]) { state.errors } else { state.errors + 1 },
                        chars_taken: state.chars_taken + 1
                    });
                    
                    // insertion
                    statestack.push(SearchState {
                        search,
                        errors: state.errors + 1,
                        chars_taken: state.chars_taken
                    });
            }}
            
            // deletion
            let search = state.search.search_backward(next_char);
            statestack.push(SearchState {
                search,
                errors: state.errors + 1,
                chars_taken: state.chars_taken + 1
            })
        }
            
        // errors not allowed
        else {
            let search = state.search.search_backward(next_char);
            if search.count() > 0 {
                statestack.push(SearchState{
                    search,
                    errors: state.errors,
                    chars_taken: state.chars_taken + 1
                });
                
            }
       }

    }
    results
}

fn main() {
    let results = searchscheme_test();
    println!("RESULTS: {:?}\n\nCount = {}", results, results.len());
}
