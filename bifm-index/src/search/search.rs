use crate::bifm_index::BiFMIndex;
use fm_index::converter::RangeConverter;
use fm_index::search::Search;
use fm_index::{BackwardIterableIndex, FMIndex};

pub struct BiFMSearch<'a> {
    pub index: &'a BiFMIndex,
    pub backward_s: u64,
    pub backward_e: u64,
    pub forward_s: u64,
    pub forward_e: u64,
}

impl<'a> BiFMSearch<'a> {
    pub fn new(index: &'a BiFMIndex) -> Self {
        Self {
            index,
            backward_s: 0,
            backward_e: index.normal_index.len() as u64,
            forward_s: 0,
            forward_e: index.normal_index.len() as u64,
            // pattern: Vec::new(),
        }
    }

    fn get_x<S>(c: u8, s: u64, e: u64, index: &FMIndex<u8, RangeConverter<u8>, S>) -> u64 {
        index.bw.rank_range_cumulative_u64_unchecked((s as usize)..(e as usize), c as u64) as u64
    }

    #[inline(always)]
    pub fn search_char(&self, char: u8, forward: bool) -> Self {
        if forward { self.search_char_forward(char) } else { self.search_char_backward(char) }
    }

    pub fn search_char_forward(&self, char: u8) -> Self {
        let index = &self.index.reverse_index;

        let s = index.lf_map2(char, self.forward_s);
        let e = index.lf_map2(char, self.forward_e);
        let other_s = self.backward_s + Self::get_x(char, self.forward_s, self.forward_e, index);
        let other_e = other_s + (e - s);

        BiFMSearch {
            index: self.index,
            backward_s: other_s,
            backward_e: other_e,
            forward_s: s,
            forward_e: e,
        }
    }
    pub fn search_char_backward(&self, char: u8) -> Self {
        let index: &FMIndex<u8, RangeConverter<u8>, _> = &self.index.normal_index;

        let s = index.lf_map2(char, self.backward_s);
        let e = index.lf_map2(char, self.backward_e);
        let other_s = self.forward_s + Self::get_x(char, self.backward_s, self.backward_e, index);
        let other_e = other_s + (e - s);

        BiFMSearch {
            index: self.index,
            backward_s: s,
            backward_e: e,
            forward_s: other_s,
            forward_e: other_e,
        }
    }

    #[inline(always)]
    pub fn search(&self, pattern: &str, forward: bool) -> Self {
        if forward { self.search_forward(pattern) } else { self.search_backward(pattern) }
    }

    pub fn search_forward(&self, pattern: &str) -> Self {
        let mut s = self.forward_s;
        let mut e = self.forward_e;
        let mut other_s = self.backward_s;
        let mut other_e = self.backward_e;
        let pattern: Vec<u8> = pattern.bytes().collect();

        let index = &self.index.reverse_index;

        for &c in pattern.iter() {
            other_s += Self::get_x(c, s, e, index);
            s = index.lf_map2(c, s);
            e = index.lf_map2(c, e);
            other_e = other_s + (e - s);
            if s == e {
                break;
            }
        }

        BiFMSearch {
            index: self.index,
            backward_s: other_s,
            backward_e: other_e,
            forward_s: s,
            forward_e: e,
        }
    }

    pub fn search_backward(&self, pattern: &str) -> Self {
        let mut s = self.backward_s;
        let mut e = self.backward_e;
        let mut other_s = self.forward_s;
        let mut other_e = self.forward_e;
        let mut pattern: Vec<u8> = pattern.bytes().collect();
        pattern.reverse();

        let index = &self.index.normal_index;

        for &c in pattern.iter() {
            other_s += Self::get_x(c, s, e, index);
            s = index.lf_map2(c, s);
            e = index.lf_map2(c, e);
            other_e = other_s + (e - s);
            if s == e {
                break;
            }
        }

        BiFMSearch {
            index: self.index,
            backward_s: s,
            backward_e: e,
            forward_s: other_s,
            forward_e: other_e,
        }
    }

    pub fn count(&self) -> u64 {
        self.backward_e - self.backward_s
    }

    pub fn locate(&self) -> Vec<u64> {
        let s = Search {
            index: &self.index.normal_index,
            s: self.backward_s,
            e: self.backward_e,
            pattern: vec![],
        };
        s.locate()
    }
}

#[cfg(test)]
mod tests {
    use crate::search::search::BiFMSearch;
    use crate::search::shared::LCG;
    use fm_index::converter::{Converter, RangeConverter};
    use fm_index::suffix_array::NullSampler;
    use fm_index::FMIndex;
    use fm_index_benchmarking::benchmarker::try_from_database_file_uncompressed_with_length;
    use std::time::Instant;

    fn get_x_control<S>(c: u8, s: u64, e: u64, index: &FMIndex<u8, RangeConverter<u8>, S>) -> u64 {
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

    fn get_test_index() -> FMIndex<u8, RangeConverter<u8>, ()> {
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
        println!("Spawned converter & sampler");
        println!("Starting building index...");
        let start_count = Instant::now();
        let index = FMIndex::new(text, converter, NullSampler::new());
        println!("Index built in {:.2}s", start_count.elapsed().as_secs_f64());
        index
    }

    #[test]
    fn get_x_correctness() {
        let test_its = 100000;
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
            let x_opt = BiFMSearch::get_x(cs[i], ss[i], es[i], &index);
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
