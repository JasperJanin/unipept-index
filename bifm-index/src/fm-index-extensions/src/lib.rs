use crate::benchmarker::{
    read_benchmark_files, run_single_benchmark, try_from_database_file_uncompressed_with_length, DatasetOption,
};
use crate::index_instances::builtin_fm_index::BuiltinFmIndex;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::FMIndex;
use postcard::{from_bytes, to_allocvec};
use sa_compression::load_compressed_suffix_array;
use sa_index::binary::load_suffix_array;
use sa_index::sa_searcher::SparseSearcher;
use sa_index::SuffixArray;
use sa_mappings::proteins::Proteins;
use std::cmp::min;
use std::error::Error;
use std::fs::write;
use std::fs::{read, File};
use std::io::{BufReader, Read};

pub mod benchmarker;
pub mod index_instances;

pub fn get_fm_converter(text: &Vec<u8>) -> RangeConverter<u8> {
    let min = *text.iter().min().unwrap();
    let max = *text.iter().max().unwrap();
    RangeConverter::new(min, max);
    RangeConverter::new(b'@', b'Z')
}

pub fn generate_fm_index_from_bytes_with_converter(
    text: Vec<u8>,
    converter: RangeConverter<u8>,
    sampling_level: usize,
) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let sampling_level = min(sampling_level, text.len() >> 10);
    let sampler = SuffixOrderSampler::new().level(sampling_level);

    FMIndex::new(text, converter, sampler)
}

pub fn convert_alphabet(text: Vec<u8>) -> Vec<u8> {
    text.into_iter()
        .map(|x| match x {
            b'-' => b'@',
            b'$' => b'@',
            _ => x.clone(),
        })
        .collect()
}

pub fn convert_alphabet_and_reverse(text: Vec<u8>) -> Vec<u8> {
    text.into_iter()
        .map(|x| match x {
            b'-' => b'@',
            b'$' => b'@',
            _ => x.clone(),
        })
        .rev()
        .collect()
}

pub fn generate_fm_index(
    inputfile: &str,
    max_length: usize,
    tsv_field: usize,
    sampling_level: usize,
) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let text =
        convert_alphabet(try_from_database_file_uncompressed_with_length(inputfile, max_length, tsv_field).unwrap());
    generate_fm_index_from_bytes_with_converter(text, RangeConverter::new(b'@', b'Z'), sampling_level)
}

pub fn generate_easy_fm_index() -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    generate_fm_index("unipept-index-data/proteins.tsv", 0, 2, 3)
}

pub fn load_easy_fm_index() -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    load_index_postcard("swissprot.postcard")
}

pub fn load_suffix_array_file(file: &str) -> Result<SuffixArray, Box<dyn Error>> {
    // Open the suffix array file
    let mut sa_file = File::open(file)?;

    // Create a buffer reader for the file
    let mut reader = BufReader::new(&mut sa_file);

    // Read the bits per value from the binary file (1 byte)
    let mut bits_per_value_buffer = [0_u8; 1];
    reader
        .read_exact(&mut bits_per_value_buffer)
        .map_err(|_| "Could not read the flags from the binary file")?;
    let bits_per_value = bits_per_value_buffer[0];

    if bits_per_value == 64 {
        load_suffix_array(&mut reader)
    } else {
        load_compressed_suffix_array(&mut reader, bits_per_value as usize)
    }
}
pub fn get_easy_sa_index() -> SparseSearcher {
    let proteins = Proteins::try_from_database_file("unipept-index-data/proteins.tsv").unwrap();
    let suffix_array = load_suffix_array_file(&"unipept-index-data/sa_sparse3_compressed.bin").unwrap();
    SparseSearcher::new(suffix_array, proteins)
}

pub fn test_memory_usage() {
    // let r = run_all_benchmarks("benchmark_patterns/", &DatasetOption::Large);

    let benchmark_dir = "benchmark_patterns/";
    // load benchmarks
    let benchmark_strings = read_benchmark_files(benchmark_dir);
    let dataset_option = &DatasetOption::Uniprot10M;

    let mut r = Vec::new();

    let mut length = 4096;

    while length < 1000000000 {
        for sampling in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] {
            let mut bm = BuiltinFmIndex::new(sampling, true, length, 6);
            r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

            println!("{:#?}", r.last());
        }

        length <<= 1;
    }
}

pub fn save_index_postcard(index: &FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>, filename: &str) {
    let postcardvec = to_allocvec(&index).expect("No postcard vec generated");
    write(filename, postcardvec).expect("Could not write file");
}

pub fn load_index_postcard(index_file: &str) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let content = read(index_file).unwrap();
    from_bytes(&content).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use fm_index::BackwardSearchIndex;
    use sa_index::sa_searcher::SearchAllSuffixesResult;

    #[test]
    pub fn test_correctness() {
        let fm_index = generate_easy_fm_index();
        let sa_searcher = get_easy_sa_index();

        let patterns = read_benchmark_files("sihumi");

        for collection in patterns {
            let mut matches = 0;
            let mut mismatches = 0;

            for s in collection.patterns.iter() {
                let mut fm_r = fm_index.search_backward(s).locate();
                let sa_search = sa_searcher.search_matching_suffixes(s.as_bytes(), 99999999, false, false);

                let sa_r = match sa_search {
                    SearchAllSuffixesResult::SearchResult(r) => r,
                    SearchAllSuffixesResult::MaxMatches(r) => r,
                    SearchAllSuffixesResult::NoMatches => Vec::new(),
                };

                let mut sa_r: Vec<u64> = sa_r.into_iter().map(|n| n as u64).collect();

                fm_r.sort();
                sa_r.sort();

                if fm_r == sa_r {
                    matches += 1;
                } else {
                    mismatches += 1;
                }
                assert_eq!(mismatches, 0);
            }
            assert_eq!(matches, collection.patterns.len());
        }
    }
}
