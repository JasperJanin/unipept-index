use std::collections::HashSet;
use crate::approximate_search::ApproximateSearch;
use crate::search::BDFMSearch;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampledArray;
use fm_index::FMIndex;
use fm_index_benchmarking::{
    generate_fm_index, generate_fm_index_from_bytes_without_known_bound, generate_reverse_fm_index,
};

pub struct BidirectionalIndex {
    pub normal_index: FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
    pub reverse_index: FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
}

impl BidirectionalIndex {
    const DEFAULT_STACK_SIZE: usize = 100_000;

    pub fn new(text: Vec<u8>, sparseness: u32) -> Self {
        let mut rev = text.clone();
        rev.reverse();
        Self {
            normal_index: generate_fm_index_from_bytes_without_known_bound(text),
            reverse_index: generate_fm_index_from_bytes_without_known_bound(rev),
        }
    }

    pub fn load() -> Self {
        BidirectionalIndex {
            normal_index: generate_fm_index("unipept-index-data/searchschemetest.txt", 0, 0),
            reverse_index: generate_reverse_fm_index("unipept-index-data/searchschemetest.txt", 0, 0),
        }
    }

    pub fn search<'a>(&self, pattern: &str, forward: bool) -> BDFMSearch {
        BDFMSearch::new(self).search(pattern, forward)
    }

    fn find_approximate_matches_config(&self, pattern: String, distance: usize, max_stack_size: usize) -> HashSet<u64> {
        ApproximateSearch::search(self, pattern, distance, max_stack_size)
    }

    pub fn find_approximate_matches(&self, pattern: String, distance: usize) -> HashSet<u64> {
        self.find_approximate_matches_config(pattern, distance, Self::DEFAULT_STACK_SIZE)
    }
}
