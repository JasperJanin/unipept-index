use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampler;
use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
use fm_index_benchmarking::{load_index_postcard, save_index_postcard};
use std::io::Result;
use clap::Parser;

fn standard_build(infile: &str, outfile: &str, size_limit: usize, tsv_field: usize) {
    println!("Starting builder...");
    println!("Loading database...");
    let text = try_from_database_file_uncompressed_with_length(infile, size_limit, tsv_field)
        .unwrap()
        .into_iter()
        .map(|x| match x {
            b'-' => b'@',
            b'$' => b'@',
            _ => x.clone(),
        })
        .collect::<Vec<u8>>();
    println!("Done loading database");
    let converter = RangeConverter::new(b'@', b'Z');
    let sampler = SuffixOrderSampler::new().level(4);
    println!("Spawned converter & sampler");
    println!("Starting building index...");
    let index = FMIndex::new(text, converter, sampler);
    println!("Index built!");
    println!("Saving to postcard binary...");
    save_index_postcard(&index, outfile);
    println!("Saved!");
    println!("All done!")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input_tsv: String,

    #[arg(short, long)]
    output_bin: String,

    #[arg(short, long, default_value_t = 0)]
    size_limit: usize,

    #[arg(short, long, default_value_t = 2)]
    tsv_field_index: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    Ok(standard_build(args.input_tsv.as_str(), args.output_bin.as_str(), args.size_limit, args.tsv_field_index))
}
