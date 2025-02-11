use std::fs;
use super::super::benchmarker::Benchmark;

use fm_index::{BackwardSearchIndex, FMIndex, RLFMIndex};
use fm_index::suffix_array::{NullSampler, SuffixOrderSampler, SuffixOrderSampledArray};
use fm_index::converter::RangeConverter;
use crate::benchmarker::{try_from_database_file_uncompressed_with_length, DatasetOption};
use crate::benchmarker::DatasetOption::{Tiny, SwissProt, Uniprot10M};

pub struct BuiltinWaveletFmIndex {
    index: Option<RLFMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>>,
    sa_sampling: usize,
    input_length: usize,
    optimize_alphabet: bool,
}

impl BuiltinWaveletFmIndex {
    pub fn new(sa_sampling: usize, optimize_alphabet: bool, input_length: usize) -> Self {
        Self { index: None, optimize_alphabet, sa_sampling, input_length }
    }

    fn internal_to_string(&self, index_rep: &str) -> String {
        if self.optimize_alphabet {
            index_rep
                .chars()
                .map(|c| match c {
                    '@' => '-',
                    _ => c.clone()
                }
                )
                .collect()
        } else {
            index_rep.to_owned()
        }
    }

    fn string_to_internal(&self, index_rep: &str) -> String {
        if self.optimize_alphabet {
            index_rep
                .chars()
                .map(|c| match c {
                    '-' => '@',
                    _ => c.clone()
                }
                )
                .collect()
        } else {
            index_rep.to_owned()
        }
    }
}

impl Benchmark for BuiltinWaveletFmIndex {

    fn build_index(&mut self, dataset_option: &DatasetOption) {
        let filepath = match dataset_option {
            Tiny => "unipept-index-data/proteins-sample.tsv",
            SwissProt => "unipept-index-data/proteins.tsv",
            Uniprot10M => "unipept-index-data/uniprot_10M.tsv",
        };

        let text = try_from_database_file_uncompressed_with_length(filepath, self.input_length).unwrap()
            .into_iter()
            .map(|x| if self.optimize_alphabet { match x {
                b'-' => b'@',
                b'$' => b'@',
                _ => x.clone()
            }
            } else {
                x.clone()
            })
            .collect::<Vec<u8>>();

        self.input_length = text.len();

        // for i in 0..500 {
        //     println!("{} - {}: {}", i, text[i], text[i] as char);
        // }

        let converter = if self.optimize_alphabet {
            RangeConverter::new(b'@', b'Z')
        } else {
            RangeConverter::new(b'$', b'Z')
        };

        // let converter = RangeConverter::new(b'\t', b'~');

        let sampler = SuffixOrderSampler::new().level(self.sa_sampling);
        self.index = Some(RLFMIndex::new(text, converter, sampler));
    }

    fn input_length(&self) -> u64 {
        self.input_length as u64
    }

    fn memory_used(&self) -> u64 {
        self.index.as_ref().unwrap().size() as u64
    }

    fn count_occurrences(&self, pattern: &str) -> u64 {
        self.index.as_ref().expect("Forgot to instantiate index?").search_backward(self.string_to_internal(pattern)).count()
    }

    fn retrieve_match_positions(&self, text: &str) -> Vec<u64> {
        self.index.as_ref().expect("Forgot to instantiate index?").search_backward(self.string_to_internal(text)).locate()
    }

    fn get_name(&self) -> String {
        format!("Built-in Run-length encoded FM Index with{} alphabet optimization, SA sampling {}{}",
                if self.optimize_alphabet {""} else {"out"}, self.sa_sampling, if self.input_length > 0 {format!(", input size {}", self.input_length)} else {"".to_string()})
    }
}
