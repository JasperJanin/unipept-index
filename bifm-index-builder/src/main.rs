#![crate_name = "bifm_index_builder"]

use clap::Parser;
use bifm_index::BiFMIndex;
use std::io::Result;


/// Builds a bidirectional FM-index into a Postcard file.
/// Use bifm-index::BiFMIndex to load the Postcard file and query the index.
///
/// # Arguments
///
/// * `database_path` - A tsv file containing protein sequences
/// * `out_path` - The path where the Postcard file is saved
/// * `tsv_field` - The index of the tsv field containing the protein sequences
/// * `sampling_level` - The sampling level of the internal suffix array (1 in 2^(sampling_level) suffixes will be stored)
fn standard_build(database_path: &str, out_path: &str, tsv_field: usize, sampling_level: usize) {
    println!("Starting builder...");
    let index = BiFMIndex::from_database_verbose(database_path, 0, sampling_level, tsv_field);
    println!("Saving to postcard binary...");
    index.save_to_postcard(out_path);
    println!("Saved!");
    println!("All done!")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    database: String,

    #[arg(short, long)]
    output_bin: String,

    #[arg(short, long, default_value_t = 2)]
    tsv_field_index: usize,

    #[arg(short, long, default_value_t = 3)]
    sampling_level: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    Ok(standard_build(args.database.as_str(), args.output_bin.as_str(), args.tsv_field_index, args.sampling_level))
}
