use super::index_instances::builtin_fm_index::BuiltinFmIndex;
use crate::{generate_easy_fm_index, get_easy_sa_index};
use bytelines::ByteLines;
use fm_index::BackwardSearchIndex;
use sa_index::sa_searcher::SearchAllSuffixesResult;
use sa_mappings::proteins::{SEPARATION_CHARACTER, TERMINATION_CHARACTER};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::from_utf8;
use std::time::Instant;

#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub input_pattern_count: u64,
    pub t_count: f64,
    pub match_count: u64,
    pub t_retrieve: f64,
}

#[derive(Debug)]
pub struct PatternCollection {
    pub name: String,
    pub patterns: Vec<String>,
}

#[derive(Debug)]
pub struct IndexBenchmark {
    index: String,
    input_length: u64,
    index_bytes: u64,
    build_t: f64,
    runs: Vec<BenchmarkResult>,
}

pub enum DatasetOption {
    Tiny,
    SwissProt,
    Uniprot10M,
}

pub fn try_from_database_file_uncompressed_with_length(
    database_file: &str,
    max_length: usize,
    tsv_field: usize,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut input_string: String = String::new();

    let file = File::open(database_file)?;

    // Read the lines as bytes, since the input string is not guaranteed to be utf8
    // because of the encoded functional annotations
    let mut lines = ByteLines::new(BufReader::new(file));

    while let Some(Ok(line)) = lines.next() {
        let mut fields = line.split(|b| *b == b'\t');

        // only get the taxon id and sequence from each line, we don't need the other parts
        let sequence = from_utf8(fields.nth(tsv_field).unwrap())?;

        input_string.push_str(&sequence.to_uppercase());
        input_string.push(SEPARATION_CHARACTER.into());
        if max_length != 0 && input_string.len() > max_length {
            break;
        }
    }

    input_string.pop();
    input_string.push(TERMINATION_CHARACTER.into());

    input_string.shrink_to_fit();
    Ok(input_string.into_bytes())
}

pub trait Benchmark {
    fn build_index(&mut self, dataset_option: &DatasetOption);

    fn input_length(&self) -> u64;
    fn memory_used(&self) -> u64;
    fn count_occurrences(&self, text: &str) -> u64;
    fn retrieve_match_positions(&self, text: &str) -> Vec<u64>;
    fn get_name(&self) -> String;
}

pub fn read_benchmark_files(benchmark_dir: &str) -> Vec<PatternCollection> {
    let mut out = Vec::new();

    let mut paths: Vec<PathBuf> = fs::read_dir(benchmark_dir).unwrap().map(|r| r.unwrap().path()).collect();
    paths.sort();

    for filepath in paths {
        if filepath.is_file() && filepath.extension().unwrap() == "txt" {
            let file = File::open(&filepath).expect("impossible: file was checked by program");
            let buf = BufReader::new(file);
            let content = buf.lines().map(|l| l.expect("Could not parse line")).collect();
            out.push(PatternCollection {
                name: filepath.file_name().unwrap().to_str().unwrap().to_string(),
                patterns: content,
            });
        }
    }
    out
}

fn run_benchmark(
    benchmark: &mut dyn Benchmark,
    input_patterns: &Vec<PatternCollection>,
    dataset_option: &DatasetOption,
) -> IndexBenchmark {
    eprintln!("Building index: {}", benchmark.get_name());

    // time building index
    let now = Instant::now();
    benchmark.build_index(dataset_option);
    let build_t = now.elapsed().as_secs_f64();

    let mut runs = Vec::new();

    for collection in input_patterns {
        eprintln!("Querying index from file: {}", collection.name);

        let start_count = Instant::now();

        let mut match_count = 0;
        for s in collection.patterns.iter() {
            match_count += benchmark.count_occurrences(s.as_str());
        }
        let t_count = start_count.elapsed().as_secs_f64();

        let mut match_count_retrieve = 0;
        let start_retrieve = Instant::now();
        for s in collection.patterns.iter() {
            match_count_retrieve += benchmark.retrieve_match_positions(s.as_str()).len() as u64;
        }
        let t_retrieve = start_retrieve.elapsed().as_secs_f64();

        if match_count != match_count_retrieve {
            println!("Match mismatch")
        }

        runs.push(BenchmarkResult {
            name: collection.name.clone(),
            input_pattern_count: collection.patterns.len() as u64,
            t_count,
            match_count,
            t_retrieve,
        });
    }

    IndexBenchmark {
        index: benchmark.get_name(),
        build_t,
        input_length: benchmark.input_length(),
        index_bytes: benchmark.memory_used(),
        runs,
    }
}

pub fn run_single_benchmark(
    benchmark: &mut dyn Benchmark,
    index_content: &Vec<PatternCollection>,
    dataset_option: &DatasetOption,
) -> IndexBenchmark {
    run_benchmark(benchmark, index_content, dataset_option)
}

pub fn run_all_benchmarks(benchmark_dir: &str, dataset_option: &DatasetOption, tsv_index: usize) {
    let mut results = Vec::new();

    // load benchmarks
    let benchmark_strings = read_benchmark_files(benchmark_dir);

    let mut bm = BuiltinFmIndex::new(0, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    let mut bm = BuiltinFmIndex::new(1, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    let mut bm = BuiltinFmIndex::new(2, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    let mut bm = BuiltinFmIndex::new(3, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    let mut bm = BuiltinFmIndex::new(4, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    let mut bm = BuiltinFmIndex::new(6, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    let mut bm = BuiltinFmIndex::new(8, false, 0, tsv_index);
    results.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    for r in results {
        println!(
            "\\multicolumn{{<SN>}}{{r|}}{{{} \\m{{SN=1}}}} & {} & {} & {} & {} \\\\",
            r.index, r.index_bytes, r.build_t, r.runs[0].t_count, r.runs[0].t_retrieve
        );
    }
}

pub fn measure_ssa(benchmark_dir: &str) {
    println!("Building suffix array index...");
    let start = Instant::now();
    let sa_searcher = get_easy_sa_index();
    println!("Suffix array ready in {:?}, reading benchmark files...", Instant::now() - start);
    let patterns = read_benchmark_files(benchmark_dir);
    println!("Benchmark ready, executing benchmark for {} datasets...", patterns.len());
    let mut sum = 0;
    let mut patt_count = 0;

    let start = Instant::now();
    for collection in &patterns {
        patt_count += collection.patterns.len();

        for s in collection.patterns.iter() {
            let sa_search = sa_searcher.search_matching_suffixes(s.as_bytes(), 99999999, false, false);

            let sa_r = match sa_search {
                SearchAllSuffixesResult::SearchResult(r) => r,
                SearchAllSuffixesResult::MaxMatches(r) => r,
                SearchAllSuffixesResult::NoMatches => Vec::new(),
            };

            let sa_r: Vec<u64> = sa_r.into_iter().map(|n| n as u64).collect();
            sum += sa_r.len();
        }
    }
    println!("SSA: {sum} occurrences found from {patt_count} patterns in {:?}", Instant::now() - start);
    println!("Generating FM-index...");
    let fm = generate_easy_fm_index();
    println!("Done, benchmarking...");
    sum = 0;
    patt_count = 0;
    let start = Instant::now();
    for collection in &patterns {
        patt_count += collection.patterns.len();

        for s in collection.patterns.iter() {
            let fm_search = fm.search_backward(s.as_bytes());
            let results = fm_search.locate();
            sum += results.len();
        }
    }
    println!("FMI: {sum} occurrences found from {patt_count} patterns in {:?}", Instant::now() - start);
}
