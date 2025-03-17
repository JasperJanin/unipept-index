use fm_index::converter::{Converter, RangeConverter};
use fm_index::{BackwardIterableIndex, FMIndex};
use fm_index::suffix_array::SuffixOrderSampledArray;
use fm_index_benchmarking::{generate_fm_index, generate_fm_index_from_bytes_without_known_bound, generate_reverse_fm_index};

pub struct BidirectionalIndex {
    pub normal_index:  FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>,
    pub reverse_index:  FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>
}

impl BidirectionalIndex {
    pub fn new(text: Vec<u8>, sparseness: u32) -> Self {
        let mut rev = text.clone();
        rev.reverse();
        Self {
            normal_index: generate_fm_index_from_bytes_without_known_bound(text),
            reverse_index: generate_fm_index_from_bytes_without_known_bound(rev),
        }
    }
}

struct SearchScheme {

}

pub struct BDFMSearch<'a> {
    index: &'a BidirectionalIndex,
    backward_s: u64,
    backward_e: u64,
    forward_s: u64,
    forward_e: u64,
    pattern: Vec<u8>,
}


impl BidirectionalIndex {
    pub fn load() -> Self {
        BidirectionalIndex {
            normal_index: generate_fm_index("unipept-index-data/searchschemetest.txt", 0, 0),
            reverse_index: generate_reverse_fm_index("unipept-index-data/searchschemetest.txt", 0, 0)
        }
    }
    
    pub fn search<'a>(&self, pattern: String, forward: bool) -> BDFMSearch {
        BDFMSearch::new(self).search(pattern, forward)
    }
}

impl<'a> BDFMSearch<'a> {
    pub fn new(index: &'a BidirectionalIndex) -> Self {
        Self {
            index,
            backward_s: 0,
            backward_e: index.normal_index.len(),
            forward_s: 0,
            forward_e: index.reverse_index.len(),
            pattern: Vec::new(),
        }
    }

    fn get_x(&self, c: u8, s: u64, e: u64, index: &FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>, forward: bool) -> u64 {

        let mut x = 0;
        
        if ('A' as u8) > c || ('Z' as u8) < c {
            panic!("Invalid character in pattern");
        }
        
        let mut c_iter = 0;
        while index.converter.convert_inv(c_iter) < c {
            x += index.bw.rank_u64_unchecked(e as usize, c_iter.into()) - index.bw.rank_u64_unchecked(s as usize, c_iter.into());
            
            c_iter += 1;
        }
        
        x as u64
    }

    pub fn search(&self, pattern: String, forward: bool) -> Self {
        let mut s = if !forward { self.backward_s } else {self.forward_s};
        let mut e = if !forward { self.backward_e } else {self.forward_e};
        let mut other_s = if forward { self.backward_s } else {self.forward_s};
        let mut other_e = if forward { self.backward_e } else {self.forward_e};
        let mut pattern: Vec<u8> = pattern.into_bytes();
        if !forward {pattern.reverse()};

        let index = if !forward {&self.index.normal_index} else {&self.index.reverse_index};

        for &c in pattern.iter() {
            s = index.lf_map2(c, s);
            e = index.lf_map2(c, e);
            if s == e {
                other_s = other_e;
                break;
            }
            other_s += self.get_x(c, s, e, index, forward);
            other_e = other_s + (e - s);
        }
        
        pattern.extend_from_slice(&self.pattern);

        BDFMSearch {
            index: self.index,
            backward_s: if !forward {s} else {other_s},
            backward_e: if !forward {e} else {other_e},
            forward_s: if forward {s} else {other_s},
            forward_e: if forward {e} else {other_e},
            pattern,
        }
    }
}
