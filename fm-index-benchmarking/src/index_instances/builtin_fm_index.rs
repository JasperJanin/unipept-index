use std::fs;
use super::super::benchmarker::Benchmark;

use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index::suffix_array::{NullSampler, SuffixOrderSampler, SuffixOrderSampledArray};
use fm_index::converter::RangeConverter;
use crate::benchmarker::DatasetOption;
use crate::benchmarker::DatasetOption::{Large, Small};

pub struct BuiltinFmIndex {
    index: Option<FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>>,
    sa_sampling: usize,
    optimize_alphabet: bool,
}

impl BuiltinFmIndex {
    pub fn new(sa_sampling: usize) -> Self {
        Self { index: None, optimize_alphabet: true, sa_sampling }
    }
}

impl Benchmark for BuiltinFmIndex {

    fn build_index(&mut self, dataset_option: &DatasetOption) {
        let filepath = match dataset_option {
            Small => "unipept-index-data/proteins-sample.tsv",
            Large => "unipept-index-data/proteins.tsv"
        };
        let text = fs::read_to_string(filepath)
            .expect("Should have been able to read the file")
            .as_bytes()
            .to_vec();

        let converter = RangeConverter::new(b'\t', b'~');

        println!("Building index...");
        let sampler = SuffixOrderSampler::new().level(self.sa_sampling);
        self.index = Some(FMIndex::new(text, converter, sampler));
        println!("Done building index");
    }

    fn input_length(&self) -> u64 {
        self.index.as_ref().expect("Forgot to instantiate index?").size() as u64
    }

    fn memory_used(&self) -> u64 {
        0
    }

    fn count_occurrences(&self, pattern: &str) -> u64 {
        self.index.as_ref().expect("Forgot to instantiate index?").search_backward(pattern).count()
    }

    fn retrieve_matches(&self, text: &str) -> Vec<String> {
        let search = self.index.as_ref().expect("Forgot to instantiate index?").search_backward(text);
        // let positions = search.locate();

        let mut results = Vec::new();

        for i in 0..search.count() {
            // get prefix
            let mut prefix = Vec::new();
            let mut iter = search.iter_backward(i);
            let mut char= iter.next();
            while char != Some(b'\n') {
                prefix.push(char.unwrap().clone());
                char = iter.next();
            }
            prefix.reverse();

            // get suffix
            let mut suffix = Vec::new();
            let mut iter = search.iter_forward(i);
            let mut char= iter.next();
            while char != Some(b'\n') {
                suffix.push(char.unwrap().clone());
                char = iter.next();
            }

            let entry = format!("{}{}{}", String::from_utf8(prefix).unwrap(), text, String::from_utf8(suffix).unwrap());
            results.push(entry);

        }
        results
    }

    fn get_name(&self) -> String {
        format!("Built-in FM Index with{} alphabet optimization, SA sampling {}",
                if self.optimize_alphabet {"out"} else {""}, self.sa_sampling)
    }
}
