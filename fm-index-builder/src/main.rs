use std::error::Error;
use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index::suffix_array::{NullSampler, SuffixOrderSampler};
use fm_index::converter::RangeConverter;

use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string("unipept-index-data/proteins.tsv")
        .expect("Should have been able to read the file")
        .as_bytes()
        .to_vec();

    // Converter converts each character into packed representation.
    // `' '` ~ `'~'` represents a range of ASCII printable characters.
    let converter = RangeConverter::new(b'\t', b'~');

    // To perform locate queries, we need to retain suffix array generated in the construction phase.
    // However, we don't need the whole array since we can interpolate missing elements in a suffix array from others.
    // A sampler will _sieve_ a suffix array for this purpose.
    // You can also use `NullSampler` if you don't perform location queries (disabled in type-level).
    // let sampler = SuffixOrderSampler::new().level(2);
    let sampler = SuffixOrderSampler::new().level(32);
    println!("Building index...");
    let index = FMIndex::new(text, converter, sampler);
    println!("Done building index");

    // Search for a pattern string.
    let pattern = "VFPGLQGGPHNHTIGGLAVCLKHAQSPEFKAYQKRVVSNC";
    println!("Searching...");
    let search = index.search_backward(pattern);
    println!("Done searching!");

    // Count the number of occurrences.
    let n = search.count();

    println!("Matches: {n}");

    // List the position of all occurrences.
    // let positions = search.locate();

    Ok(())
}