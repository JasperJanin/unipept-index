use std::time::Instant;
use fm_index_benchmarking::benchmarker::read_benchmark_files;
use fm_searcher::bd_index::BidirectionalIndex;

fn benchmark() {
    #[derive(Debug)]
    struct BenchmarkResult {
        pub name: String,
        pub input_pattern_count: u64,
        pub t_retrieve: f64,
        pub match_count: usize,
        pub distance: usize,
        pub index_size: usize,
    }
    const DATA_FILE: &str = "unipept-index-data/proteins.tsv";
    
    let mut size = 1000;
    let mut prev_index_size = 0;
    
    println!("Building index with size {}...", size);
    let mut index = BidirectionalIndex::from_file_max_length(DATA_FILE, size, 2);
    println!("Index built");
    
    while index.normal_index.len() != prev_index_size {
        let test_patterns = read_benchmark_files("benchmark_patterns");


        for distance in 0..3 {

            for collection in test_patterns.iter() {
                eprintln!("Querying index from file: {} | Size {} | Dist {}", collection.name, size, distance);

                let start_count = Instant::now();
                let pattern_ct = collection.patterns.len();

                let mut match_count = 0;
                for s in collection.patterns.iter() {
                    match_count += index.find_approximate_matches(s.clone(), distance).len();
                }
                let t_retrieve = start_count.elapsed().as_secs_f64();

                println!("{:#?}", BenchmarkResult {
                    name: collection.name.clone(),
                    input_pattern_count: pattern_ct as u64,
                    t_retrieve,
                    match_count,
                    distance,
                    index_size: size
                });
            }
        }
        
        prev_index_size = index.normal_index.len();
        size <<= 1;
        println!("Building index with size {}...", size);
        index = BidirectionalIndex::from_file_max_length(DATA_FILE, size, 2);
        println!("Index built");
    }
    
    
}

fn main() {
    benchmark();
}
