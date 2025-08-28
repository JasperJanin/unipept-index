use super::super::benchmarker::Benchmark;

use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index::suffix_array::{SuffixOrderSampler, SuffixOrderSampledArray};
use fm_index::converter::RangeConverter;
use crate::benchmarker::{try_from_database_file_uncompressed_with_length, DatasetOption};
use crate::benchmarker::DatasetOption::{SwissProt, Tiny, Uniprot10M};

pub struct BuiltinFmIndex {
    index: Option<FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>>,
    input_length: usize,
    sa_sampling: usize,
    optimize_alphabet: bool,
    tsv_index: usize,
}

impl BuiltinFmIndex {
    pub fn new(sa_sampling: usize, optimize_alphabet: bool, input_length: usize, tsv_index: usize) -> Self {
        Self { index: None, optimize_alphabet, sa_sampling, input_length, tsv_index }
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

impl Benchmark for BuiltinFmIndex {

    fn build_index(&mut self, dataset_option: &DatasetOption) {
        let filepath = match dataset_option {
            Tiny => "unipept-index-data/proteins-sample.tsv",
            SwissProt => "unipept-index-data/proteins.tsv",
            Uniprot10M => "unipept-index-data/uniprot_10M.tsv",
        };
        
        let text = try_from_database_file_uncompressed_with_length(filepath, self.input_length, self.tsv_index).unwrap()
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
        
        // for i in 0..500 {
        //     println!("{} - {}: {}", i, text[i], text[i] as char);
        // }
        
        self.input_length = text.len();

        let converter = if self.optimize_alphabet {
            RangeConverter::new(b'@', b'Z')
        } else {
            RangeConverter::new(b'$', b'Z')
        };

        // let converter = RangeConverter::new(b'\t', b'~');

        let sampler = SuffixOrderSampler::new().level(self.sa_sampling);
        self.index = Some(FMIndex::new(text, converter, sampler));
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
        format!("Built-in FM Index with{} alphabet optimization, SA sampling {}{}",
                if self.optimize_alphabet {""} else {"out"}, self.sa_sampling, if self.input_length > 0 {format!(", input size {}", self.input_length)} else {"".to_string()})
    }
}
