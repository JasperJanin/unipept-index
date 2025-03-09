use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampler;
use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
use fm_index_benchmarking::{load_index_postcard, save_index_postcard, test_correctness};

fn main() {
    test_correctness();
}
