use crate::search::search::BiFMSearch;
use crate::search::search_dynamic::ApproximateDynamicSearch;
use crate::search::search_stack::ApproximateStackSearch;
use crate::search::shared::SearchMethod;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{NullSampler, SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::FMIndex;
use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
use fm_index_benchmarking::{convert_alphabet, get_fm_converter};
use std::cmp::min;
use std::fs::{read, write};
use postcard::{from_bytes, to_allocvec};
use serde::{Serialize,Deserialize};



/// The representation of a bidirectional FM-index
#[derive(Serialize, Deserialize)]
pub struct BiFMIndex {
    /// Regular FM-index of the text, containing a suffix array to locate results in the text
    pub normal_index: FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
    /// Complementary FM-index of the reversed text without a suffix array, allowing bidirectional search
    pub reverse_index: FMIndex<u8, RangeConverter<u8>, ()>,
}


impl BiFMIndex {
    /// Returns a BiFMIndex from any text given any sampling level.
    /// The text is not required to contain a protein sequence.
    ///
    /// # Arguments
    /// * `text` - The string being indexed, given in ASCII bytes
    /// * `sampling_level` - The sampling level of the internal suffix array (1 in 2^(sampling_level) suffixes will be stored)
    pub fn new(text: Vec<u8>, sampling_level: usize) -> Self {
        let sampling_level = min(sampling_level, text.len() >> 10);
        let mut rev_text = text.clone();
        rev_text.reverse();

        let converter = get_fm_converter(&text);

        let sampler = SuffixOrderSampler::new().level(sampling_level);

        let normal_index = FMIndex::new(text, converter.clone(), sampler);
        let reverse_index = FMIndex::new(rev_text, converter, NullSampler::new());

        Self { normal_index, reverse_index }
    }

    /// Returns a BiFMIndex from a protein sequence file, with a given maximum length.
    /// The database file can be a TSV file, in which case the index of the field containing
    /// the sequences must be specified.
    ///
    /// # Arguments
    /// * `database_path` - A tsv file containing protein sequences
    /// * `max_length` - Once the concatenated proteins reach this length, no more are added to the index. Set to zero for unlimited size.
    /// * `sampling_level` - The sampling level of the internal suffix array (1 in 2^(sampling_level) suffixes will be stored)
    /// * `tsv_field` - The index of the tsv field containing the protein sequences (otherwise zero)
    pub fn from_database_max_length(database_path: &str, max_length: usize, sampling_level: usize, tsv_field: usize) -> Self {
        let text =
            convert_alphabet(try_from_database_file_uncompressed_with_length(database_path, max_length, tsv_field).unwrap());
        Self::new(text, sampling_level)
    }

    /// Returns a BiFMIndex from a protein sequence file.
    /// The database file can be a TSV file, in which case the index of the field containing
    /// the sequences must be specified.
    ///
    /// # Arguments
    /// * `database_path` - A tsv file containing protein sequences
    /// * `sampling_level` - The sampling level of the internal suffix array (1 in 2^(sampling_level) suffixes will be stored)
    /// * `tsv_field` - The index of the tsv field containing the protein sequences (otherwise zero)
    pub fn from_database(database_path: &str, sampling_level: usize, tsv_field: usize) -> Self {
        Self::from_database_max_length(database_path, 0, sampling_level, tsv_field)
    }

    /// Returns a BiFMIndex from a protein sequence file while logging basic updates to stdout.
    /// A maximum length can be specified for the text.
    /// The database file can be a TSV file, in which case the index of the field containing
    /// the sequences must be specified.
    ///
    /// # Arguments
    /// * `database_path` - A tsv file containing protein sequences
    /// * `max_length` - Once the concatenated proteins reach this length, no more are added to the index. Set to zero for unlimited size.
    /// * `sampling_level` - The sampling level of the internal suffix array (1 in 2^(sampling_level) suffixes will be stored)
    /// * `tsv_field` - The index of the tsv field containing the protein sequences (otherwise zero)
    pub fn from_database_verbose(database_path: &str, max_length: usize, sampling_level: usize, tsv_field: usize) -> Self {
        println!("Loading database...");
        let text =
            convert_alphabet(try_from_database_file_uncompressed_with_length(database_path, max_length, tsv_field).unwrap());
        println!("Done loading database");
        println!("Starting building index...");
        let index = Self::new(text, sampling_level);
        println!("Index built!");
        index
    }

    /// Saves a BiFMIndex object to a binary file in the Postcard format.
    /// Use `BiFMIndex::load_from_postcard` to load the Postcard file.
    ///
    /// # Arguments
    /// * `filepath` - A path to the desired output location
    pub fn save_to_postcard(&self, filepath: &str) {
        let postcard_vec = to_allocvec(&self).expect("No postcard vec generated");
        write(filepath, postcard_vec).expect("Could not write file");
    }

    /// Loads a BiFMIndex object from a binary file in the Postcard format.
    /// Use `BiFMIndex::save_to_postcard` to save a Postcard file.
    ///
    /// # Arguments
    /// * `filepath` - A path to the desired output location
    pub fn load_from_postcard(filepath: &str) -> Self {
        let content = read(filepath).unwrap();
        from_bytes(&content).unwrap()
    }

    /// Perform a search using the bidirectional FM-index.
    /// This method only retrieves exact matches.
    ///
    /// # Arguments
    /// * `pattern` - The pattern to search
    /// 
    /// # Returns
    /// A `BiFMSearch` object representing the search.
    /// Call `locate` on the object to retrieve the matches or extend the search in any direction
    /// by calling `search` on the object.
    pub fn search(&self, pattern: &str) -> BiFMSearch {
        BiFMSearch::new(self).search(pattern, false)
    }

    /// Perform an approximate search using the bidirectional FM-index.
    /// This method retrieves approximate matches that have a maximally allowed edit distance.
    ///
    /// # Arguments
    /// * `pattern` - The pattern to search
    /// * `distance` - The maximally allowed edit distance between the pattern and text matches
    ///
    /// # Returns
    /// A `BiFMSearch` object representing the search.
    /// Call `locate` on the object to retrieve the matches or extend the search in any direction
    /// by calling `search` on the object.
    pub fn search_approx(&self, pattern: String, distance: usize) -> Vec<u64> {
        self.search_approx_with_method(pattern, distance, &SearchMethod::Dynamic)
    }

    /// Perform an approximate search using the bidirectional FM-index,
    /// using a specified search algorithm.
    /// This method retrieves approximate matches that have a maximally allowed edit distance.
    ///
    /// # Arguments
    /// * `pattern` - The pattern to search
    /// * `distance` - The maximally allowed edit distance between the pattern and text matches
    /// * `search_method` - Either `SearchMethod::Stack` or `SearchMethod::Dynamic`,
    /// for an ad-hoc algorithm or a dynamic programming algorithm
    /// based on the Needleman & Wunsch algorithm.
    /// Use `BiFMIndex::search_approx` when in doubt.
    ///
    /// # Returns
    /// A `BiFMSearch` object representing the search.
    /// Call `locate` on the object to retrieve the matches or extend the search in any direction
    /// by calling `search` on the object.
    pub fn search_approx_with_method(&self, pattern: String, dist: usize, search_method: &SearchMethod) -> Vec<u64> {
        match search_method {
            SearchMethod::Stack => ApproximateStackSearch::search(self, pattern, dist),
            SearchMethod::Dynamic => ApproximateDynamicSearch::search(self, pattern, dist),
        }
    }

    /// Retrieve the length of the text indexed by this BiFMIndex.
    pub fn len(&self) -> u64 {
        self.normal_index.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::shared::LCG;
    use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
    use fm_index_benchmarking::convert_alphabet;

    #[test]
    fn short_searches() {
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
    }

    #[test]
    fn random_test() {
        const MIN_PATTERN_LENGTH: usize = 16;
        const MAX_PATTERN_LENGTH: usize = 1024;
        const TEST_ITERATIONS: usize = 1000;

        /// Modify a pattern by either substituting, deleting or inserting a random character.
        /// The edit distance between the original pattern and resulting pattern is one.
        /// 
        /// # Arguments
        /// * `pattern` - The pattern to modify
        /// * `rng` - A seeded random number generator
        fn edit_pattern(pattern: &mut Vec<u8>, rng: &mut LCG) {
            let choice = rng.get(3);
            let patt_len = pattern.len();
            match choice {
                // substitution
                0 => {
                    pattern[rng.get(patt_len)] = *ApproximateDynamicSearch::ALPHABET
                        .iter()
                        .nth(rng.get(ApproximateDynamicSearch::ALPHABET.len()))
                        .unwrap()
                }
                // insertion
                1 => pattern.insert(
                    rng.get(patt_len),
                    *ApproximateDynamicSearch::ALPHABET
                        .iter()
                        .nth(rng.get(ApproximateDynamicSearch::ALPHABET.len()))
                        .unwrap(),
                ),
                // deletion
                2 => {
                    pattern.remove(rng.get(patt_len));
                }
                _ => panic!("Impossible branch"),
            };
        }

        // Init random generator, text and bifm-index
        let mut rng = LCG::from_seed(0);
        let text = convert_alphabet(
            try_from_database_file_uncompressed_with_length("test-data/testproteins.tsv", 0, 2).unwrap(),
        );
        let index = BiFMIndex::from_database("test-data/testproteins.tsv", 3, 2);

        let text_len = text.len();

        let mut right = 0;
        let mut wrong = 0;

        
        for i in 0..TEST_ITERATIONS {
            let pattern_length = rng.get(MAX_PATTERN_LENGTH - MIN_PATTERN_LENGTH) + MIN_PATTERN_LENGTH;
            let start_pos = rng.get(text_len - pattern_length);

            let mut pattern = text[start_pos..start_pos + pattern_length].to_vec();
            let mut last_round_results = vec![start_pos as u64];

            // Each pattern is modified `dist` times
            for dist in 0..3 {
                if dist >> 5 > pattern_length {
                    break;
                }

                let pattern_string = String::from_utf8(pattern.clone()).unwrap();

                let this_round_results = index.search_approx_with_method(pattern_string, dist, &SearchMethod::Dynamic);
                assert!(this_round_results.len() > 0);

                // Each time, check that previous results are still present in searches for modified strings
                // with a greater edit distance
                for result in &last_round_results {
                    if !(this_round_results.contains(result) || this_round_results.contains(&(*result + 1))) {
                        wrong += 1;
                        println!("(dist {dist}) Expected result {result} in list: {:?}", &this_round_results);
                        // break;
                    }
                    right += 1;

                    assert!(
                        this_round_results.contains(&result) || this_round_results.contains(&(result + 1)),
                        "Editing search did not yield all expected values"
                    );
                }

                edit_pattern(&mut pattern, &mut rng);

                last_round_results = this_round_results;
            }
        }
        println!("Right: {right}\nWrong: {wrong}");
    }
}
