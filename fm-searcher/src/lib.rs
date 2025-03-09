use fm_index::converter::RangeConverter;
use fm_index::FMIndex;
use fm_index::suffix_array::SuffixOrderSampledArray;
use fm_index_benchmarking::{generate_fm_index, generate_reverse_fm_index};

struct BidirectionalIndex {
    normal_index:  FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
    reverse_index:  FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>
}

struct BDFMSearch {
    
}


impl BidirectionalIndex {
    pub fn load() -> Self {
        BidirectionalIndex {
            normal_index: generate_fm_index("unipept-index-data/dnatext.txt", 0, 0),
            reverse_index: generate_reverse_fm_index("unipept-index-data/dnatext.txt", 0, 0)
        }
    }
    
    pub fn search(&self, pattern)
}

