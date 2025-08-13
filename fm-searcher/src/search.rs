use crate::bd_index::BidirectionalIndex;
use fm_index::converter::{Converter, RangeConverter};
use fm_index::suffix_array::{IndexWithSA, SuffixOrderSampledArray};
use fm_index::{BackwardIterableIndex, FMIndex};

pub struct BDFMSearch<'a> {
    pub index: &'a BidirectionalIndex,
    pub backward_s: u64,
    pub backward_e: u64,
    pub forward_s: u64,
    pub forward_e: u64,
}

impl<'a> BDFMSearch<'a> {
    pub fn new(index: &'a BidirectionalIndex) -> Self {
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
        &self,
        c: u8,
        s: u64,
        e: u64,
        index: &FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
        forward: bool,
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

    pub fn search_char(&self, char: u8, forward: bool) -> Self {
        let mut s = if !forward { self.backward_s } else { self.forward_s };
        let mut e = if !forward { self.backward_e } else { self.forward_e };
        let mut other_s = if forward { self.backward_s } else { self.forward_s };

        let index = if !forward { &self.index.normal_index } else { &self.index.reverse_index };

        other_s += self.get_x(char, s, e, index, forward);
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
            other_s += self.get_x(c, s, e, index, forward);
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
