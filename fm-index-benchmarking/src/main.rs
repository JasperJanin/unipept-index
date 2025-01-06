use std::env;
use fm_index_benchmarking::benchmarker::{run_all_benchmarks, DatasetOption};

fn main() {

    let r = run_all_benchmarks("/Users/jasper/UGent/mp/unipept-index/sihumi", &DatasetOption::Large);

    println!("{r:#?}");
}
