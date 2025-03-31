use std::collections::HashSet;
use crate::bd_index::BidirectionalIndex;
use crate::search::BDFMSearch;
use crate::search_scheme::SearchScheme;
use std::str;
use std::string::ToString;
use fm_index::BackwardSearchIndex;
use rand::prelude::*;

struct SearchStateEntry<'a> {
    search: BDFMSearch<'a>,
    errors: u8,
    chars_taken: usize,
    scheme_part_index: usize,
    forward: bool,
    remaining_string_slice: &'a str,
}

pub(crate) struct SearchSchemePass {
    pub(crate) order: Vec<u32>,
    pub(crate) lower: Vec<u32>,
    pub(crate) upper: Vec<u32>,
}

pub struct ApproximateSearch<'a> {
    index: &'a BidirectionalIndex,
    parts: &'a Vec<String>,
    pattern_length: usize,
    scheme_pass: &'a SearchSchemePass,
    results: HashSet<u64>,
    stack: Vec<SearchStateEntry<'a>>,
    max_stack_size: usize,
}

impl<'a> ApproximateSearch<'a> {
    // const ALPHABET: &'static [u8] = "ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    pub const ALPHABET: &'static [u8] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes(); // TODO verander naar echt alfabet ^

    pub fn search(index: &BidirectionalIndex, pattern: String, dist: usize, max_stack_size: usize) -> Vec<u64> {

        if pattern.len() <= dist {
            panic!("Pattern length must be greater than edit distance");
        }

        // normal search when dist == 0
        if dist <= 0 {
            return index.normal_index.search_backward(pattern).locate();
        }

        let scheme = SearchScheme::new(dist);

        let part_ct = scheme.passes[0].order.len();
        let chars_per_part = pattern.len() / part_ct;
        let mut parts = Vec::new();
        for i in 0..part_ct - 1 {
            parts.push(pattern[i * chars_per_part..(i + 1) * chars_per_part].to_string());
        }
        parts.push(pattern[(part_ct - 1) * chars_per_part..pattern.len()].to_string());

        let mut results = HashSet::new();

        for i in 0..scheme.pass_count {
            let mut search = ApproximateSearch {
                index,
                parts: &parts,
                pattern_length: pattern.len(),
                scheme_pass: &scheme.passes[i as usize],
                results: HashSet::new(),
                stack: Vec::new(),
                max_stack_size,
            };

            search.stack.push(SearchStateEntry {
                search: BDFMSearch::new(&index),
                errors: 0,
                chars_taken: 0,
                scheme_part_index: 0,
                forward: false,
                remaining_string_slice: parts[scheme.passes[i as usize].order[0] as usize].as_str(),
            });

            while search.stack.len() > 0 {
                let state = search.stack.pop().unwrap();

                // check for total result
                if state.chars_taken >= search.pattern_length {
                    state.search.locate().iter().for_each(|r| { search.results.insert(*r); });
                    continue;
                }

                search.extend_search(state);
            }
            search.results.iter().for_each(|r| { results.insert(*r); });
        }
        Vec::from_iter(results)
    }

    fn extend_search(&mut self, mut state: SearchStateEntry<'a>) {
        if state.remaining_string_slice.len() == 0 {
            state.scheme_part_index += 1;
            state.remaining_string_slice =
                self.parts[self.scheme_pass.order[state.scheme_part_index] as usize].as_str();
            state.forward = self.scheme_pass.order[state.scheme_part_index] > self.scheme_pass.order[0];
        }

        let permitted_mistakes = self.scheme_pass.upper[state.scheme_part_index];

        let next_char = if state.forward { // todo move within errors allowed
            state.remaining_string_slice.bytes().next().unwrap()
        } else {
            state.remaining_string_slice.bytes().last().unwrap()
        };
        
        // errors allowed
        if state.errors < permitted_mistakes as u8 {
            for c in Self::ALPHABET {
                let mut search = BDFMSearch {
                    index: state.search.index,
                    backward_s: state.search.backward_s,
                    backward_e: state.search.backward_e,
                    forward_s: state.search.forward_s,
                    forward_e: state.search.forward_e,
                    // pattern: state.search.pattern.clone(),
                };
                search = search.search_char(*c, state.forward);

                if search.count() > 0 {
                    // match/mismatch
                    self.stack.push(SearchStateEntry {
                        search: BDFMSearch {
                            index: search.index,
                            backward_s: search.backward_s,
                            backward_e: search.backward_e,
                            forward_s: search.forward_s,
                            forward_e: search.forward_e,
                            // pattern: search.pattern.clone(),
                        },
                        errors: if *c == next_char { state.errors } else { state.errors + 1 },
                        chars_taken: state.chars_taken + 1,
                        scheme_part_index: state.scheme_part_index,
                        forward: state.forward,
                        remaining_string_slice: if state.forward {
                            &state.remaining_string_slice[1..]
                        } else {
                            &state.remaining_string_slice[..state.remaining_string_slice.len() - 1]
                        },
                    });

                    // insertion
                    self.stack.push(SearchStateEntry {
                        search,
                        errors: state.errors + 1,
                        chars_taken: state.chars_taken,
                        scheme_part_index: state.scheme_part_index,
                        forward: state.forward,
                        remaining_string_slice: state.remaining_string_slice,
                    });
                }
            }

            // deletion
            self.stack.push(SearchStateEntry {
                search: state.search,
                errors: state.errors + 1,
                chars_taken: state.chars_taken + 1,
                scheme_part_index: state.scheme_part_index,
                forward: state.forward,
                remaining_string_slice: if state.forward {
                    &state.remaining_string_slice[1..]
                } else {
                    &state.remaining_string_slice[..state.remaining_string_slice.len() - 1]
                },
            });
        }
        // errors not allowed
        else {
            let search = state.search.search(state.remaining_string_slice, state.forward);
            if search.count() > 0 {
                state.search = search;
                state.chars_taken += state.remaining_string_slice.len();
                state.remaining_string_slice = "";
                self.stack.push(state);
            }
        }
    }
}

pub fn searchscheme_test() -> Vec<u64> {
    let index = BidirectionalIndex::from_str("AABBCCDDEEDDCCBBAA");

    let search_term = "CBB".to_string();

    let results = index.find_approximate_matches(search_term, 1);
    results
}

#[cfg(test)]
mod tests {
    use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
    use rand::random;
    use fm_index_benchmarking::convert_alphabet;
    use super::*;

    #[test]
    fn short_text() {
        let text = "A";
        let index = BidirectionalIndex::from_str(text);
        let search_string = "ACD";

        let results = index.find_approximate_matches(search_string.to_string(), 0);
        assert_eq!(results.len(), 0);
        let results = index.find_approximate_matches(search_string.to_string(), 1);
        assert_eq!(results.len(), 0);
        let results = index.find_approximate_matches(search_string.to_string(), 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }

    #[test]
    fn short_searches_1() {
        let text = "DANANA";
        let index = BidirectionalIndex::from_str(text);

        let search_string = "ANA";
        let results = index.find_approximate_matches(search_string.to_string(), 0);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&3));

        let search_string = "NANA".to_string();
        let results = index.find_approximate_matches(search_string.to_string(), 1);
        assert_eq!(results.len(), 4);
        assert!(results.contains(&0));
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));

        let results = index.find_approximate_matches(search_string.to_string(), 2);
        assert_eq!(results.len(), 5);
        assert!(results.contains(&0));
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));
        assert!(results.contains(&4));
    }

    #[test]
    fn short_searches_2() {
        let text = "GARFIELDTHELASTFATCAT";
        let index = BidirectionalIndex::from_str(text);

        let search_string = "CAT";
        let results = index.find_approximate_matches(search_string.to_string(), 0);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&18));

        let results = index.find_approximate_matches(search_string.to_string(), 1);
        assert_eq!(results.len(), 4);
        assert!(results.contains(&15));
        assert!(results.contains(&16));
        assert!(results.contains(&18));
        assert!(results.contains(&19));

        let search_string = "LAST";
        let results = index.find_approximate_matches(search_string.to_string(), 2);
        assert_eq!(results.len(), 9);
        assert!(results.contains(&6));
        assert!(results.contains(&10));
        assert!(results.contains(&11));
        assert!(results.contains(&12));
        assert!(results.contains(&13));
        assert!(results.contains(&15));
        assert!(results.contains(&16));
        assert!(results.contains(&18));
        assert!(results.contains(&19));
    }

    #[test]
    fn random_test() {
        struct LCG { // https://rosettacode.org/wiki/Linear_congruential_generator#Rust
            state: u32
        }
        impl LCG {
            fn get(&mut self, upper: usize) -> usize {
                self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                self.state %= 1 << 31;
                self.state as usize % upper
            }
            fn from_seed(seed: u32) -> Self {
                Self { state: seed }
            }
        }

        fn edit_pattern(pattern: &mut Vec<u8>, rng: &mut LCG) {
            let choice = rng.get(3);
            let patt_len = pattern.len();
            match choice {
                // substitution
                0 => pattern[rng.get(patt_len)] = *ApproximateSearch::ALPHABET.iter().nth(rng.get(ApproximateSearch::ALPHABET.len())).unwrap(),
                // insertion
                1 => pattern.insert(rng.get(patt_len), *ApproximateSearch::ALPHABET.iter().nth(rng.get(ApproximateSearch::ALPHABET.len())).unwrap()),
                // deletion
                2 => {pattern.remove(rng.get(patt_len));},
                _ => panic!("Impossible branch")
            };
        }
        
        let mut rng = LCG::from_seed(0);
        let mut pattern = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        for i in 0..10 {
            edit_pattern(&mut pattern, &mut rng);
        }
        
        const MIN_PATTERN_LENGTH: usize = 16;
        const MAX_PATTERN_LENGTH: usize = 1024;
        const TEST_ITERATIONS: usize = 1000;


        let text = convert_alphabet(try_from_database_file_uncompressed_with_length("test-data/testproteins.tsv", 0, 2).unwrap());
        let index = BidirectionalIndex::from_file("test-data/testproteins.tsv", 2);

        let text_len = text.len();


        for _ in 0..TEST_ITERATIONS {
            let pattern_length = rng.get(MAX_PATTERN_LENGTH - MIN_PATTERN_LENGTH) + MIN_PATTERN_LENGTH;
            let start_pos = rng.get(text_len - pattern_length);
            
            let mut pattern = text[start_pos..start_pos + pattern_length].to_vec();
            let mut last_round_results = Vec::new();
            
            for dist in 0..3 {
                if dist >> 3 > pattern_length {
                    break;
                }
                let this_round_results = index.find_approximate_matches(String::from_utf8(pattern.clone()).unwrap(), dist);
                
                for result in last_round_results {
                    assert!(this_round_results.contains(&result), "Editing search did not yield all expected values");
                }
                
                edit_pattern(&mut pattern, &mut rng);
                
                last_round_results = this_round_results;
                
            }
        }
    }
}
