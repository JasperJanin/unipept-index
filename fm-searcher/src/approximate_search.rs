use crate::bd_index::BidirectionalIndex;
use std::string::ToString;
use crate::search_methods::search_dynamic::ApproximateDynamicSearch;
use crate::search_methods::search_stack::ApproximateStackSearch;
use crate::search_methods::shared::SearchMethod;

pub fn searchscheme_test() -> Vec<u64> {
    let index = BidirectionalIndex::from_str("AABBCCDDEEDDCCBBAA");

    let search_term = "CBB".to_string();

    let results = index.find_approximate_matches(search_term, 1);
    results
}

pub struct ApproximateSearch {}

impl ApproximateSearch {
    pub fn search(index: &BidirectionalIndex, pattern: String, dist: usize, max_stack_size: usize) -> Vec<u64> {
        ApproximateSearch::search_with_method(index, pattern, dist, max_stack_size, SearchMethod::Stack)
    }
    
    pub fn search_with_method(index: &BidirectionalIndex, pattern: String, dist: usize, max_stack_size: usize, search_method: SearchMethod) -> Vec<u64> {
        match search_method { 
            SearchMethod::Stack => ApproximateStackSearch::search(index, pattern, dist, max_stack_size),
            SearchMethod::Dynamic => ApproximateDynamicSearch::search(index, pattern, dist),
        }
    }
}

#[cfg(test)]
mod tests {
    use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
    use rand::random;
    use fm_index_benchmarking::convert_alphabet;
    use crate::search_methods::search_stack::ApproximateStackSearch;
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
        const MIN_PATTERN_LENGTH: usize = 16;
        const MAX_PATTERN_LENGTH: usize = 1024;
        const TEST_ITERATIONS: usize = 1000;
        
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
                0 => pattern[rng.get(patt_len)] = *ApproximateStackSearch::ALPHABET.iter().nth(rng.get(ApproximateStackSearch::ALPHABET.len())).unwrap(),
                // insertion
                1 => pattern.insert(rng.get(patt_len), *ApproximateStackSearch::ALPHABET.iter().nth(rng.get(ApproximateStackSearch::ALPHABET.len())).unwrap()),
                // deletion
                2 => {pattern.remove(rng.get(patt_len));},
                _ => panic!("Impossible branch")
            };
        }
        
        let mut rng = LCG::from_seed(0);

        let text = convert_alphabet(try_from_database_file_uncompressed_with_length("test-data/testproteins.tsv", 0, 2).unwrap());
        let index = BidirectionalIndex::from_file("test-data/testproteins.tsv", 2);

        let text_len = text.len();
        
        let mut right = 0;
        let mut wrong = 0;


        for i in 0..TEST_ITERATIONS {
            let pattern_length = rng.get(MAX_PATTERN_LENGTH - MIN_PATTERN_LENGTH) + MIN_PATTERN_LENGTH;
            let start_pos = rng.get(text_len - pattern_length);
            
            let mut pattern = text[start_pos..start_pos + pattern_length].to_vec();
            let mut last_round_results = vec![start_pos as u64];
            
            for dist in 0..3 {
                if dist >> 5 > pattern_length {
                    break;
                }
                
                let pattern_string = String::from_utf8(pattern.clone()).unwrap();

                let this_round_results = index.find_approximate_matches_method(pattern_string, dist, SearchMethod::Stack);
                assert!(this_round_results.len() > 0);
                
                for result in &last_round_results {
                    if !(this_round_results.contains(result) || this_round_results.contains(&(*result+1))) {
                        wrong += 1;
                        println!("(dist {dist}) Expected result {result} in list: {:?}", &this_round_results);
                        // break;
                    }
                    right += 1;
                    
                    assert!(this_round_results.contains(&result) || this_round_results.contains(&(result+1)), "Editing search did not yield all expected values");
                }
                
                edit_pattern(&mut pattern, &mut rng);
                
                last_round_results = this_round_results;
                
            }
        }
        println!("Right: {right}\nWrong: {wrong}");
    }
}
