use crate::benchmarker::{
    read_benchmark_files, run_single_benchmark, try_from_database_file_uncompressed_with_length, DatasetOption,
};
use crate::index_instances::builtin_fm_index::BuiltinFmIndex;
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::{BackwardSearchIndex, FMIndex};
use postcard::{from_bytes, to_allocvec};
use sa_compression::load_compressed_suffix_array;
use sa_index::binary::load_suffix_array;
use sa_index::sa_searcher::{SearchAllSuffixesResult, SparseSearcher};
use sa_index::SuffixArray;
use sa_mappings::proteins::Proteins;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{read, File};
use std::fs::write;
use std::io::prelude::*;
use std::io::{BufReader, Read, Write};

pub mod benchmarker;
pub mod index_instances;

pub fn generate_fm_index_from_bytes_with_known_bound(text: Vec<u8>, min_char: u8, max_char: u8) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let converter = RangeConverter::new(min_char, max_char);
    let sampler = SuffixOrderSampler::new().level(4);
    FMIndex::new(text, converter, sampler)
}

pub fn generate_fm_index_from_bytes_without_known_bound(text: Vec<u8>) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let min = *text.iter().min().unwrap();
    let max = *text.iter().max().unwrap();
    generate_fm_index_from_bytes_with_known_bound(text, min, max)
}

pub fn generate_fm_index(inputfile: &str, max_length: usize, tsv_field: usize) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let text = try_from_database_file_uncompressed_with_length(inputfile, max_length, tsv_field)
        .unwrap()
        .into_iter()
        .map(|x| match x {
            b'-' => b'@',
            b'$' => b'@',
            _ => x.clone(),
        })
        .collect::<Vec<u8>>();
    generate_fm_index_from_bytes_with_known_bound(text, b'@', b'Z')
}

pub fn generate_reverse_fm_index(inputfile: &str, max_length: usize, tsv_field: usize) -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    let text = try_from_database_file_uncompressed_with_length(inputfile, max_length, tsv_field)
        .unwrap()
        .into_iter()
        .rev()
        .map(|x| match x {
            b'-' => b'@',
            b'$' => b'@',
            _ => x.clone(),
        })
        .collect::<Vec<u8>>();
    generate_fm_index_from_bytes_with_known_bound(text, b'@', b'Z')
}

pub fn generate_easy_fm_index() -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    generate_fm_index("unipept-index-data/proteins.tsv", 0, 2)
}

pub fn load_easy_fm_index() -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
    load_index_postcard("swissprot.postcard")
}


pub fn eprint_and_exit(err: &str) -> ! {
    eprintln!("{}", err);
    std::process::exit(1);
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
pub fn test_correctness() {
    fn array_eq_unordered(arr1: &Vec<u64>, arr2: &Vec<u64>) -> bool {
        let mut arr1_copy = arr1.to_owned();
        let mut arr2_copy = arr2.to_owned();

        arr1_copy.sort();
        arr2_copy.sort();

        arr1_copy == arr2_copy
    }
    fn string_to_internal(index_rep: &str) -> String {
        index_rep
            .chars()
            .map(|c| match c {
                '-' => '@',
                _ => c.clone(),
            })
            .collect()
    }
    let fm_index = generate_easy_fm_index();
    let sa_searcher = get_easy_sa_index();

    let patterns = read_benchmark_files("sihumi");

    let mut total_matches = 0;
    let mut total_mismatches = 0;

    for collection in patterns {
        println!("Testing patterns from file: {}", collection.name);

        let mut matches = 0;
        let mut mismatches = 0;

        for s in collection.patterns.iter() {
            let fm_r = fm_index.search_backward(string_to_internal(s)).locate();
            let sa_search = sa_searcher.search_matching_suffixes(s.as_bytes(), 99999999, false, false);

            let sa_r = match sa_search {
                SearchAllSuffixesResult::SearchResult(r) => r,
                SearchAllSuffixesResult::MaxMatches(r) => r,
                SearchAllSuffixesResult::NoMatches => Vec::new(),
            };

            let sa_r = sa_r.into_iter().map(|n| n as _).collect();

            if array_eq_unordered(&sa_r, &fm_r) {
                matches += 1;
            } else {
                mismatches += 1;
            }
        }

        println!("{} matches, {} inaccuracies found", matches, mismatches);
        total_matches += matches;
        total_mismatches += mismatches;
    }
    println!("Done! In total: {} matches, {} inaccuracies found", total_matches, total_mismatches);
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
    #[test]
    fn it_works() {
        assert_eq!(1, 1);
    }
}
