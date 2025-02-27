use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampler;
use fm_index::{BackwardSearchIndex, FMIndex};
use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
use fm_index_benchmarking::{load_index_postcard, save_index_postcard};

fn main() {
    let text = try_from_database_file_uncompressed_with_length("unipept-index-data/proteins.tsv", 0)
        .unwrap()
        .into_iter()
        .map(|x| match x {
            b'-' => b'@',
            b'$' => b'@',
            _ => x.clone(),
        })
        .collect::<Vec<u8>>();
    let converter = RangeConverter::new(b'@', b'Z');
    let sampler = SuffixOrderSampler::new().level(4);
    let index = FMIndex::new(text, converter, sampler);
    save_index_postcard(&index, "sortofbigindex.postcard");
    
    let index = load_index_postcard("sortofbigindex.postcard");
    println!("{}", index.search_backward("SVAF").count());
}
