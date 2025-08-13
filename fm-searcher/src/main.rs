use std::time::Instant;
use fm_index_benchmarking::benchmarker::read_benchmark_files;
use fm_searcher::bd_index::BidirectionalIndex;
use fm_searcher::search_methods::shared::SearchMethod;

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

fn execute_dyn_search() {
    const DATA_FILE: &str = "fm-searcher/test-data/testproteins.tsv";

    println!("Building index...");
    // let mut index = BidirectionalIndex::from_file_max_length(DATA_FILE, size, 2);
    let mut index = BidirectionalIndex::from_file(DATA_FILE, 2);
    println!("Index built");
    // let s = "ILKKRWVLELSMIAGIDPQSEGLARARAEGVY".to_string();
    // KGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKAD
    let s = "VLEYARHKRKLRLGALKGNAFTLVLREVSNRDDVEQRLIDICVKGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKADVNQVVDGDALQLAGRGSWFVATTEELAELRRVNDKVLMITAVLPGSGEWGTQREALAFEQAAVAEETELQTLLVREKVEAARRAMLLYPQQLSWNWWDDVTVEIRFWLPAGSFATSVVRELINTTGDYAHIAE@MIEFDNLTYLHGKPQGTGLLKANPE".to_string();
    // let s = "VLEYARHKRKLRLGALKGNAFTLVLREVSNRDDVEQRLIDICVKGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKADVNQVVDGDALQLAGRGSWFVATTEELAELQRRVNDKVLMITAVLPGSGEWGTQREALAFEQAAVAEETELQTLLVREKVEAARRAMLLYPQQLSWNWWDDVTVEIRFWLPAGSFATSVVRELINTTGDYAHIAE@MIEFDNLTYLHGKPQGTGLLKANPE".to_string();
    // let s = "VLEYARHKRKLRLGALKGNAFTLVLREVSNRDDVEQRLIDICVKGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKADVNQVVDGDALQLAGRGSWFVATTEELAELQRRVNDK".to_string();
    // let s = "VLEYARHKRKLRLGDALKGNAFTCLVLREVSNRDD".to_string();
    let mut res = index.find_approximate_matches_method(s, 1, SearchMethod::Dynamic);
    res.sort();
    for r in res {
        println!("{:#?}", r);
    }
}


fn main() {
    // benchmark()
    execute_dyn_search();
}
