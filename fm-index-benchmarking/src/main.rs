use std::env;
use fm_index_benchmarking::benchmarker::{read_benchmark_files, run_all_benchmarks, run_single_benchmark, DatasetOption};
use fm_index_benchmarking::index_instances::builtin_fm_index::BuiltinFmIndex;
use fm_index_benchmarking::index_instances::builtin_wavelet_fm::BuiltinWaveletFmIndex;

fn main() {

    // let r = run_all_benchmarks("benchmark_patterns/", &DatasetOption::Large);
    
    let benchmark_dir = "benchmark_patterns/";
    // load benchmarks
    let benchmark_strings = read_benchmark_files(benchmark_dir);
    let dataset_option = &DatasetOption::Large;

    let mut r = Vec::new();
    
    let mut bm = BuiltinWaveletFmIndex::new(1, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinWaveletFmIndex::new(2, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinWaveletFmIndex::new(4, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinWaveletFmIndex::new(6, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinWaveletFmIndex::new(8, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinFmIndex::new(1, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinFmIndex::new(2, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinFmIndex::new(4, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinFmIndex::new(6, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

    let mut bm = BuiltinFmIndex::new(8, true);
    r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
    
    println!("{r:#?}");
}
