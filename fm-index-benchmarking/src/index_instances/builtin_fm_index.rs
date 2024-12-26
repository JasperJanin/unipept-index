use std::fs;
use super::super::benchmarker::Benchmark;

use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index::suffix_array::{NullSampler, SuffixOrderSampler, SuffixOrderSampledArray};
use fm_index::converter::RangeConverter;

pub struct BuiltinFmIndex {
    index: Option<FMIndex<u8, RangeConverter<u8>, ()>>
}

impl BuiltinFmIndex {
    pub fn new() -> Self {
        Self { index: None }
    }
}

impl Benchmark for BuiltinFmIndex {

    fn build_index(&mut self) {
        let text = fs::read_to_string("/home/jasper/ugent/masterproef/unipept-index/unipept-index-data/proteins.tsv")
        .expect("Should have been able to read the file")
        .as_bytes()
        .to_vec();

        let converter = RangeConverter::new(b'\t', b'~');
        // let sampler = SuffixOrderSampler::new().level(32);
        let sampler = NullSampler::new();
        println!("Building index...");
        self.index = Some(FMIndex::new(text, converter, sampler));
        println!("Done building index");
    }

    fn count_occurrences(&self, pattern: &str) -> u64 {
        self.index.as_ref().expect("Forgot to instantiate index?").search_backward(pattern).count()
    }

    fn retrieve_matches(&self, text: &str) -> Vec<String> {
        todo!()
    }

    fn get_name(&self) -> String {
        "Built-in FM Index without alphabet optimization or sparse suffix array".to_string()
    }
}
