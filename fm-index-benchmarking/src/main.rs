use fm_index_benchmarking::benchmarker::{read_benchmark_files, run_all_benchmarks, run_single_benchmark, DatasetOption};
use fm_index_benchmarking::index_instances::builtin_fm_index::BuiltinFmIndex;
use fm_index_benchmarking::index_instances::builtin_wavelet_fm::BuiltinWaveletFmIndex;

fn main() {

    // let r = run_all_benchmarks("benchmark_patterns/", &DatasetOption::Large);
    
    let benchmark_dir = "benchmark_patterns/";
    // load benchmarks
    let benchmark_strings = read_benchmark_files(benchmark_dir);
    let dataset_option = &DatasetOption::Uniprot10M;

    let mut r = Vec::new();
    
    let mut length = 4096;
    
    while length < 1000000000 {
        
        for sampling in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] {
            let mut bm = BuiltinFmIndex::new(sampling, true, length);
            r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));
            
            println!("{:#?}", r.last());
            
            
            let mut bm = BuiltinWaveletFmIndex::new(sampling, true, length);
            r.push(run_single_benchmark(&mut bm, &benchmark_strings, dataset_option));

            println!("{:#?}", r.last());
        }
        
        length <<= 1;
    }

    // println!("{r:#?}");
}
