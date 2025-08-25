use std::ops::Range;
use crate::bd_index::{BiFMIndex};
use fm_index::converter::{RangeConverter};
use fm_index::suffix_array::{IndexWithSA, SuffixOrderSampledArray};
use fm_index::{BackwardIterableIndex, FMIndex};

pub struct BDFMSearch<'a> {
    pub index: &'a BiFMIndex,
    pub backward_s: u64,
    pub backward_e: u64,
    pub forward_s: u64,
    pub forward_e: u64,
}

impl<'a> BDFMSearch<'a> {
    pub fn new(index: &'a BiFMIndex) -> Self {
        Self {
            index,
            backward_s: 0,
            backward_e: index.normal_index.len(),
            forward_s: 0,
            forward_e: index.reverse_index.len(),
            // pattern: Vec::new(),
        }
    }

    fn get_x(
        c: u8,
        s: u64,
        e: u64,
        index: &FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
    ) -> u64 {
        index.bw.rank_range_cumulative_u64_unchecked((s as usize)..(e as usize), c as u64) as u64
    }

    pub fn search_char(&self, char: u8, forward: bool) -> Self {
        let mut s = if !forward { self.backward_s } else { self.forward_s };
        let mut e = if !forward { self.backward_e } else { self.forward_e };
        let mut other_s = if forward { self.backward_s } else { self.forward_s };

        let index = if !forward { &self.index.normal_index } else { &self.index.reverse_index };

        other_s += Self::get_x(char, s, e, index);
        s = index.lf_map2(char, s);
        e = index.lf_map2(char, e);
        let other_e = other_s + (e - s);

        // let mut pattern = Vec::new();
        // if forward {
        //     pattern.extend_from_slice(self.pattern.as_slice());
        //     pattern.push(char);
        // } else {
        //     pattern.push(char);
        //     pattern.extend_from_slice(self.pattern.as_slice());
        // }

        BDFMSearch {
            index: self.index,
            backward_s: if !forward { s } else { other_s },
            backward_e: if !forward { e } else { other_e },
            forward_s: if forward { s } else { other_s },
            forward_e: if forward { e } else { other_e },
            // pattern,
        }
    }

    pub fn search(&self, pattern: &str, forward: bool) -> Self {
        let mut s = if !forward { self.backward_s } else { self.forward_s };
        let mut e = if !forward { self.backward_e } else { self.forward_e };
        let mut other_s = if forward { self.backward_s } else { self.forward_s };
        let mut other_e = if forward { self.backward_e } else { self.forward_e };
        let mut pattern: Vec<u8> = pattern.bytes().collect();
        if !forward {
            pattern.reverse()
        };

        let index = if !forward { &self.index.normal_index } else { &self.index.reverse_index };

        for &c in pattern.iter() {
            other_s += Self::get_x(c, s, e, index);
            s = index.lf_map2(c, s);
            e = index.lf_map2(c, e);
            other_e = other_s + (e - s);
            if s == e {
                break;
            }
        }

        // if !forward {
        //     pattern.reverse();
        //     pattern.extend_from_slice(&self.pattern);
        // } else {
        //     let saved_pattern = pattern.clone();
        //     pattern = self.pattern.clone();
        //     pattern.extend_from_slice(&saved_pattern);
        // }

        BDFMSearch {
            index: self.index,
            backward_s: if !forward { s } else { other_s },
            backward_e: if !forward { e } else { other_e },
            forward_s: if forward { s } else { other_s },
            forward_e: if forward { e } else { other_e },
            // pattern,
        }
    }

    pub fn count(&self) -> u64 {
        self.backward_e - self.backward_s
    }

    pub fn locate(&self) -> Vec<u64> {
        let mut results: Vec<u64> = Vec::with_capacity((self.backward_e - self.backward_s) as usize);
        for k in self.backward_s..self.backward_e {
            results.push(self.index.normal_index.get_sa(k));
        }
        results
    }
}


#[cfg(test)]
mod tests {
    use std::time::{Instant};
    use fm_index::converter::{Converter, RangeConverter};
    use fm_index::FMIndex;
    use fm_index::suffix_array::{SuffixOrderSampledArray, SuffixOrderSampler};
    use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
    use crate::search::search::BDFMSearch;
    use crate::search::shared::LCG;

    fn get_x_control(
        c: u8,
        s: u64,
        e: u64,
        index: &FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>
    ) -> u64 {
        let mut x = 0;

        if ('@' as u8) > c || ('Z' as u8) < c {
            panic!("Invalid character in pattern");
        }

        let mut c_iter = 0;
        while index.converter.convert_inv(c_iter) < c {
            let rank_s = index.bw.rank_u64_unchecked(s as usize, c_iter.into());
            let rank_e = index.bw.rank_u64_unchecked(e as usize, c_iter.into());
            x += rank_e - rank_s;
            c_iter += 1;
        }
        x as u64
    }

    fn get_test_index() -> FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray> {
        println!("Loading database...");
        let text = try_from_database_file_uncompressed_with_length("test-data/testproteins.tsv", 0, 2)
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
        let sampler = SuffixOrderSampler::new().level(16);
        println!("Spawned converter & sampler");
        println!("Starting building index...");
        let start_count = Instant::now();
        let index = FMIndex::new(text, converter, sampler);
        println!("Index built in {:.2}s", start_count.elapsed().as_secs_f64());
        index
    }

    #[test]
    fn get_x_correctness() {
        let test_its = 1000000;
        let index = get_test_index();

        // collections
        let mut controls = Vec::new();
        controls.reserve(test_its);
        let mut opts = Vec::new();
        opts.reserve(test_its);
        let mut cs = Vec::new();
        let mut ss = Vec::new();
        let mut es = Vec::new();

        // get random parameters for searches
        let fm_size = index.bw.len();
        let mut rng = LCG::from_seed(0);
        for _ in 0..test_its {
            cs.push(index.converter.convert_inv(rng.get_char_i()));
            let pattern_size = rng.get(200) + 100;
            let s = rng.get(fm_size - pattern_size) as u64;
            ss.push(s);
            es.push(s + pattern_size as u64);
        }

        // benchmark control
        let start = Instant::now();
        for i in 0..test_its {
            let x_control = get_x_control(cs[i], ss[i], es[i], &index);
            controls.push(x_control);
        }
        let control_t = start.elapsed().as_secs_f64();

        // benchmark opt
        let start = Instant::now();
        for i in 0..test_its {
            let x_opt = BDFMSearch::get_x(cs[i], ss[i], es[i], &index);
            opts.push(x_opt);
        }
        let opt_t = start.elapsed().as_secs_f64();

        // check correctness
        for i in 0..test_its {
            assert_eq!(controls[i], opts[i]);
        }

        println!("\nSuccessfully tested {test_its} calculations of x");
        println!("Time for control: {:.2}s", control_t);
        println!("Time for optimized method: {:.2}s", opt_t);
    }
}
