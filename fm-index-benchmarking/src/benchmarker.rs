use std::time::{Instant, SystemTime};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::pattern::Pattern;
use super::index_instances::builtin_fm_index::{BuiltinFmIndex};

#[derive(Debug)]
pub struct BenchmarkResult {
    pub t_start: SystemTime,
    pub t_limit: u64,
    pub t_build: f64,
    pub m_build: f64,
    pub t_count_s03: f64,
    pub t_count_s05: f64,
    pub t_count_s07: f64,
    pub t_count_s08: f64,
    pub t_count_s11: f64,
    pub t_count_s14: f64,
    pub t_count_all: f64,
    pub t_retrieve_s03: f64,
    pub t_retrieve_s05: f64,
    pub t_retrieve_s07: f64,
    pub t_retrieve_s08: f64,
    pub t_retrieve_s11: f64,
    pub t_retrieve_s14: f64,
    pub t_retrieve_all: f64,
    pub total_count: u64,
}

impl BenchmarkResult {
    fn new() -> Self {
        BenchmarkResult {
            t_start: SystemTime::now(),
            t_limit: 0,
            t_build: 0.,
            m_build: 0.,
            t_count_s03: 0.,
            t_count_s05: 0.,
            t_count_s07: 0.,
            t_count_s08: 0.,
            t_count_s11: 0.,
            t_count_s14: 0.,
            t_count_all: 0.,
            t_retrieve_s03: 0.,
            t_retrieve_s05: 0.,
            t_retrieve_s07: 0.,
            t_retrieve_s08: 0.,
            t_retrieve_s11: 0.,
            t_retrieve_s14: 0.,
            t_retrieve_all: 0.,
            total_count: 0,
        }
    }
}

struct PatternCollection {
    name: String,
    patterns: Vec<String>,
}


pub trait Benchmark {
    fn build_index(&mut self);
    fn count_occurrences(&self, text: &str) -> u64;
    fn retrieve_matches(&self, text: &str) -> Vec<String>;
    fn get_name(&self) -> String;
}

fn read_benchmark_files(benchmark_dir: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();

    for nr in ["03", "05", "07", "08", "11", "14"] {
        let filename = format!("{benchmark_dir}/S{nr}.txt");
        let file = File::open(filename).expect("no such file");
        let buf = BufReader::new(file);
        out.push(buf.lines()
            .map(|l| l.expect("Could not parse line"))
            .collect());
    }

    out
}

fn run_benchmark(benchmark: &mut dyn Benchmark, benchmark_strings: &Vec<PatternCollection>) -> BenchmarkResult {


    let mut result = BenchmarkResult::new();

    // time building index
    let now = Instant::now();
    benchmark.build_index();
    result.t_build = now.elapsed().as_secs_f64();
    
    
    let now = Instant::now();
    for collection in benchmark_strings {
        for s in collection.patterns.iter() {
            result.total_count += benchmark.count_occurrences(s.as_str());
        }
    }

    // time
    result.t_count_all = now.elapsed().as_secs_f64();

    result
}

pub fn run_single_benchmark(benchmark: &mut dyn Benchmark, benchmark_dir: &str) -> BenchmarkResult {

    // load benchmarks
    let benchmark_strings = read_benchmark_files(benchmark_dir);

    run_benchmark(benchmark, &benchmark_strings)

}

pub fn run_all_benchmarks(benchmark_dir: &str) -> Vec<BenchmarkResult> {
    let mut results = Vec::new();
    
    let mut bm = BuiltinFmIndex::new();
    results.push(run_single_benchmark(&mut bm, benchmark_dir));
    

    results
}
