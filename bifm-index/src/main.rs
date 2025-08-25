use std::fs::File;
use std::io::BufReader;
use std::str::from_utf8;
use std::time::Instant;
use fm_index_benchmarking::benchmarker::read_benchmark_files;
use fm_searcher::bd_index::BiFMIndex;
use fm_searcher::search::shared::SearchMethod;
use bytelines::ByteLines;

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
    let mut index = BiFMIndex::from_file_max_length(DATA_FILE, size, 2);
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
        index = BiFMIndex::from_file_max_length(DATA_FILE, size, 2);
        println!("Index built");
    }
    
    
}

fn execute_dyn_search() {
    const DATA_FILE: &str = "bifm-index/test-data/testproteins.tsv";

    println!("Building index...");
    // let mut index = BidirectionalIndex::from_file_max_length(DATA_FILE, size, 2);
    let mut index = BiFMIndex::from_file(DATA_FILE, 2);
    println!("Index built");
    // let s = "ILKKRWVLELSMIAGIDPQSEGLARARAEGVY".to_string();
    // KGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKAD
    let s = "VLEYARHKRKLRLGALKGNAFTLVLREVSNRDDVEQRLIDICVKGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKADVNQVVDGDALQLAGRGSWFVATTEELAELRRVNDKVLMITAVLPGSGEWGTQREALAFEQAAVAEETELQTLLVREKVEAARRAMLLYPQQLSWNWWDDVTVEIRFWLPAGSFATSVVRELINTTGDYAHIAE@MIEFDNLTYLHGKPQGTGLLKANPE".to_string();
    // let s = "VLEYARHKRKLRLGALKGNAFTLVLREVSNRDDVEQRLIDICVKGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKADVNQVVDGDALQLAGRGSWFVATTEELAELQRRVNDKVLMITAVLPGSGEWGTQREALAFEQAAVAEETELQTLLVREKVEAARRAMLLYPQQLSWNWWDDVTVEIRFWLPAGSFATSVVRELINTTGDYAHIAE@MIEFDNLTYLHGKPQGTGLLKANPE".to_string();
    // let s = "VLEYARHKRKLRLGALKGNAFTLVLREVSNRDDVEQRLIDICVKGVPNYFGAQRFGIGGSNLQGALRWAQTNTPVRDRNKRSFWLSAARSALFNQIVAERLKKADVNQVVDGDALQLAGRGSWFVATTEELAELQRRVNDK".to_string();
    // let s = "VLEYARHKRKLRLGDALKGNAFTCLVLREVSNRDD".to_string();
    let mut res = index.find_approximate_matches_method(s, 1, &SearchMethod::Dynamic);
    res.sort();
    for r in res {
        println!("{:#?}", r);
    }
}

struct LCG { // https://rosettacode.org/wiki/Linear_congruential_generator#Rust
    state: u32
}
impl LCG {
    fn get(&mut self, upper: usize) -> usize {
        self.state = self.state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        self.state %= 1 << 31;
        self.state as usize % upper
    }
    fn from_seed(seed: u32) -> Self {
        Self { state: seed }
    }
}


fn get_search_patterns_of_length(length: usize, amount: usize, tsv_file: &str, tsv_field: usize) -> Vec<String> {
    let mut patterns = Vec::new();

    let file = File::open(tsv_file).unwrap();

    // Read the lines as bytes, since the input string is not guaranteed to be utf8
    // because of the encoded functional annotations
    let mut lines = ByteLines::new(BufReader::new(file));

    while let Some(Ok(line)) = lines.next() {
        let mut fields = line.split(|b| *b == b'\t');

        // only get the taxon id and sequence from each line, we don't need the other parts
        let sequence = from_utf8(fields.nth(tsv_field).unwrap()).unwrap();
        patterns.push(sequence.to_uppercase().to_string());
    }
    
    let mut rg = LCG::from_seed((length * amount) as u32);
    let mut results = Vec::new();
    let mut i = 0;
    while i < amount {
        let p_i = rg.get(patterns.len());
        let p = &patterns[p_i];
        if p.len() >= length {
            let pattern_start = rg.get(p.len() - length + 1);
            results.push(p[pattern_start..pattern_start + length].to_string());
            i += 1;
        }
    }
    results
}

fn benchmark_indexes() {
    const DATA_FILE: &str = "unipept-index-data/proteins.tsv";

    println!("Building index...");
    let mut index = BiFMIndex::from_file(DATA_FILE, 2);
    println!("Index built in ...");

    for pattern_length in [10, 20, 25, 50, 100, 200, 300] {
        println!("p = {pattern_length}");
        let r = get_search_patterns_of_length(pattern_length, 1000, "bifm-index/test-data/testproteins.tsv", 2);
        for distance in 0..4 {
            if pattern_length / (distance+1) < distance {continue;}
            
            for method in [SearchMethod::Dynamic] {
                eprintln!("Querying index with distance {} and method {}", distance, if method == SearchMethod::Dynamic { "dynamic" } else { "stack" });
                
                let start_count = Instant::now();
        
                let mut match_count = 0;
        
                for s in &r {
                    match_count += index.find_approximate_matches_method(s.clone(), distance, &method).len();
                }
                
                let t_retrieve = start_count.elapsed().as_secs_f64();
                println!("{match_count} matches in {} seconds", t_retrieve);
            }
        }
    }

}


fn main() {
    // benchmark()
    benchmark_indexes();
}
