use std::cmp::min;
use crate::search::search::BiFMSearch;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{NullSampler, SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::{FMIndex};
use fm_index_benchmarking::{convert_alphabet, get_fm_converter};
use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
use crate::search::search_dynamic::ApproximateDynamicSearch;
use crate::search::search_stack::ApproximateStackSearch;
use crate::search::shared::{SearchMethod};

pub struct BiFMIndex {
    pub normal_index: FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
    pub reverse_index: FMIndex<u8, RangeConverter<u8>, ()>,
}

impl BiFMIndex {

    pub fn new(text: Vec<u8>, sampling_level: usize) -> Self {
        let sampling_level = min(sampling_level, text.len() >> 10);
        let mut rev_text = text.clone();
        rev_text.reverse();
        
        let converter = get_fm_converter(&text);

        let sampler = SuffixOrderSampler::new().level(sampling_level);
        
        let normal_index = FMIndex::new(text, converter.clone(), sampler);
        let reverse_index = FMIndex::new(rev_text, converter, NullSampler::new());
        
        Self {
            normal_index,
            reverse_index,
        }
    }
    
    pub fn from_file_max_length(filepath: &str, max_length: usize, tsv_field: usize) -> Self {
        let text = convert_alphabet(try_from_database_file_uncompressed_with_length(filepath, max_length, tsv_field)
        .unwrap());
        Self::new(text, 3)
    }
    
    pub fn from_file(filename: &str, tsv_field: usize) -> Self {
        Self::from_file_max_length(filename, 0, tsv_field)
    }

    pub fn search(&self, pattern: &str, forward: bool) -> BiFMSearch {
        BiFMSearch::new(self).search(pattern, forward)
    }

    pub fn search_approx(&self, pattern: String, distance: usize) -> Vec<u64> {
        self.search_approx_with_method(pattern, distance, &SearchMethod::Dynamic)
    }

    pub fn search_approx_with_method(&self, pattern: String, dist: usize, search_method: &SearchMethod) -> Vec<u64> {
        match search_method {
            SearchMethod::Stack => ApproximateStackSearch::search(self, pattern, dist),
            SearchMethod::Dynamic => ApproximateDynamicSearch::search(self, pattern, dist),
        }
    }
    
    pub fn len(&self) -> u64{
        self.normal_index.len()
    }
}

#[cfg(test)]
mod tests {
    use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
    use fm_index_benchmarking::convert_alphabet;
    use super::*;
    use crate::search::shared::LCG;

    #[test]
    fn short_text() {
        let text = "A";
        let index = BiFMIndex::new(text.as_bytes().to_vec(), 0);
        let search_string = "ACD";

        let results = index.search_approx(search_string.to_string(), 0);
        assert_eq!(results.len(), 0);
        let results = index.search_approx(search_string.to_string(), 1);
        assert_eq!(results.len(), 0);
        let results = index.search_approx(search_string.to_string(), 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], 0);
    }

    #[test]
    fn short_searches_1() {
        let text = "DANANA";
        let index = BiFMIndex::new(text.as_bytes().to_vec(), 0);

        let search_string = "ANA";
        let results = index.search_approx(search_string.to_string(), 0);
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&3));

        let search_string = "NANA".to_string();
        let results = index.search_approx(search_string.to_string(), 1);
        assert_eq!(results.len(), 4);
        assert!(results.contains(&0));
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        assert!(results.contains(&3));

        let results = index.search_approx(search_string.to_string(), 2);
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
        let index = BiFMIndex::new(text.as_bytes().to_vec(), 0);

        let search_string = "CAT";
        let results = index.search_approx(search_string.to_string(), 0);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&18));

        let results = index.search_approx(search_string.to_string(), 1);
        assert_eq!(results.len(), 4);
        assert!(results.contains(&15));
        assert!(results.contains(&16));
        assert!(results.contains(&18));
        assert!(results.contains(&19));

        let search_string = "LAST";
        let results = index.search_approx(search_string.to_string(), 2);
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


        fn edit_pattern(pattern: &mut Vec<u8>, rng: &mut LCG) {
            let choice = rng.get(3);
            let patt_len = pattern.len();
            match choice {
                // substitution
                0 => pattern[rng.get(patt_len)] = *ApproximateDynamicSearch::ALPHABET.iter().nth(rng.get(ApproximateDynamicSearch::ALPHABET.len())).unwrap(),
                // insertion
                1 => pattern.insert(rng.get(patt_len), *ApproximateDynamicSearch::ALPHABET.iter().nth(rng.get(ApproximateDynamicSearch::ALPHABET.len())).unwrap()),
                // deletion
                2 => {pattern.remove(rng.get(patt_len));},
                _ => panic!("Impossible branch")
            };
        }

        let mut rng = LCG::from_seed(0);

        let text = convert_alphabet(try_from_database_file_uncompressed_with_length("test-data/testproteins.tsv", 0, 2).unwrap());
        let index = BiFMIndex::from_file("test-data/testproteins.tsv", 2);

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

                let this_round_results = index.search_approx_with_method(pattern_string, dist, &SearchMethod::Dynamic);
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
