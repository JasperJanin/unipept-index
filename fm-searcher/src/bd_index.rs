use std::collections::HashSet;
use crate::approximate_search::ApproximateSearch;
use crate::search::BDFMSearch;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampledArray;
use fm_index::FMIndex;
use fm_index_benchmarking::{convert_alphabet, generate_fm_index, generate_fm_index_from_bytes_with_known_bound, generate_fm_index_from_bytes_without_known_bound, generate_reverse_fm_index};

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
    
    pub fn from_file_max_length(filename: &str, max_length: usize, tsv_field: usize) -> Self {
        BidirectionalIndex {
            normal_index: generate_fm_index(filename, max_length, tsv_field),
            reverse_index: generate_reverse_fm_index(filename, max_length, tsv_field),
        }
    }
    
    pub fn from_file(filename: &str, tsv_field: usize) -> Self {
        Self::from_file_max_length(filename, 0, tsv_field)
    }

    pub fn from_str(text: &str) -> Self {
        
        let text_normal = text.bytes().collect::<Vec<_>>();
        let text_rev = text.bytes().rev().collect::<Vec<_>>();
        
        BidirectionalIndex {
            normal_index: generate_fm_index_from_bytes_with_known_bound(convert_alphabet(text_normal), b'@', b'Z'),
            reverse_index: generate_fm_index_from_bytes_with_known_bound(convert_alphabet(text_rev), b'@', b'Z')
        }
    }

    pub fn search<'a>(&self, pattern: &str, forward: bool) -> BDFMSearch {
        BDFMSearch::new(self).search(pattern, forward)
    }

    fn find_approximate_matches_config(&self, pattern: String, distance: usize, max_stack_size: usize) -> Vec<u64> {
        ApproximateSearch::search(self, pattern, distance, max_stack_size)
    }

    pub fn find_approximate_matches(&self, pattern: String, distance: usize) -> Vec<u64> {
        self.find_approximate_matches_config(pattern, distance, Self::DEFAULT_STACK_SIZE)
    }
}
