use fm_index_benchmarking::benchmarker::run_all_benchmarks;

fn main() {
    let r = run_all_benchmarks("/home/jasper/ugent/masterproef/unipept-index/unipept-index-data/sihumi");

    println!("{r:#?}");
}