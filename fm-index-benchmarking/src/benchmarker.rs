use std::time::{Instant, SystemTime};
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use std::ops::Index;
use std::path::PathBuf;
use super::index_instances::builtin_fm_index::{BuiltinFmIndex};

#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub input_pattern_count: u64,
    pub t_count: f64,
    pub match_count: u64,
    pub t_retrieve: f64,
}

#[derive(Debug)]
struct PatternCollection {
    name: String,
    patterns: Vec<String>,
}

#[derive(Debug)]
pub struct IndexBenchmark {
    index: String,
    input_length: u64,
    index_mb: u64,
    build_t: f64,
    runs: Vec<BenchmarkResult>,
}

pub enum DatasetOption {
    Small,
    Large,
}

pub trait Benchmark {
    fn build_index(&mut self, dataset_option: &DatasetOption);
    fn input_length(&self) -> u64;
    fn memory_used(&self) -> u64;
    fn count_occurrences(&self, text: &str) -> u64;
    fn retrieve_matches(&self, text: &str) -> Vec<String>;
    fn get_name(&self) -> String;
}

fn read_benchmark_files(benchmark_dir: &str) -> Vec<PatternCollection> {
    let mut out = Vec::new();

    let mut paths: Vec<PathBuf> = fs::read_dir(benchmark_dir).unwrap()
        .map(|r| r.unwrap().path())
        .collect();
    paths.sort();

    for filepath in paths {
        if filepath.is_file() && filepath.extension().unwrap() != "zip" {
            let file = File::open(&filepath).expect("impossible: file was checked by program");
            let buf = BufReader::new(file);
            let content = buf.lines()
                .map(|l| l.expect("Could not parse line"))
                .collect();
            out.push(PatternCollection {
                name: filepath.file_name().unwrap().to_str().unwrap().to_string(),
                patterns: content,
            } );
        }
    }
    out
}

fn run_benchmark(benchmark: &mut dyn Benchmark, input_patterns: &Vec<PatternCollection>, dataset_option: &DatasetOption) -> IndexBenchmark {

    // time building index
    let now = Instant::now();
    benchmark.build_index(dataset_option);
    let build_t = now.elapsed().as_secs_f64();

    let mut runs = Vec::new();

    for collection in input_patterns {
        let start_count = Instant::now();

        let mut match_count = 0;
        for s in collection.patterns.iter() {
            match_count += benchmark.count_occurrences(s.as_str());
        }
        let t_count = start_count.elapsed().as_secs_f64();

        let mut match_count_retrieve = 0;
        let start_retrieve = Instant::now();
        let mut i = 0;
        for s in collection.patterns.iter() {
            if i % 500 == 0 {
                println!("{}", i);
            }
            i += 1;
            match_count_retrieve += benchmark.retrieve_matches(s.as_str()).len() as u64;
        }
        let t_retrieve = start_retrieve.elapsed().as_secs_f64();
        println!("HEY");

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
        index_mb: benchmark.memory_used(),
        runs,
    }
}

pub fn run_single_benchmark(benchmark: &mut dyn Benchmark, benchmark_dir: &str, dataset_option: &DatasetOption) -> IndexBenchmark {

    // load benchmarks
    let benchmark_strings = read_benchmark_files(benchmark_dir);

    run_benchmark(benchmark, &benchmark_strings, dataset_option)

}

pub fn run_all_benchmarks(benchmark_dir: &str, dataset_option: &DatasetOption) -> Vec<IndexBenchmark> {
    let mut results = Vec::new();

    let mut bm = BuiltinFmIndex::new(1);
    results.push(run_single_benchmark(&mut bm, benchmark_dir, dataset_option));

    let mut bm = BuiltinFmIndex::new(32);
    results.push(run_single_benchmark(&mut bm, benchmark_dir, dataset_option));

    let mut bm = BuiltinFmIndex::new(128);
    results.push(run_single_benchmark(&mut bm, benchmark_dir, dataset_option));


    results
}
