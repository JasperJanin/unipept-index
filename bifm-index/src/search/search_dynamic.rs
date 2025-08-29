use crate::bifm_index::BiFMIndex;
use crate::search::banded_matrix::BandedMatrix;
use crate::search::search::BiFMSearch;
use crate::search_scheme::{SearchScheme, SearchSchemePass};
use fm_index::BackwardSearchIndex;
use std::collections::HashSet;
// todo acknowledgement

pub struct SearchDepthChar<'a> {
    // bifmext
    pub search: BiFMSearch<'a>,
    pub depth: u32,
    pub char: u8,
}

pub struct SearchDepthDist<'a> {
    // bifmocc
    pub search: BiFMSearch<'a>,
    pub depth: u32,
    pub dist: u32,
}

pub struct PassDetails {
    pub order: Vec<u32>,
    pub lower: Vec<u32>,
    pub upper: Vec<u32>,
    pub forward: Vec<bool>,
}

pub struct ApproximateDynamicSearch<'i, 's> {
    index: &'i BiFMIndex,
    pattern: &'s str,
    pattern_split: Vec<&'s str>,
    scheme: SearchScheme,
    forward: bool,
    results: HashSet<u64>,
}

impl<'i, 's> ApproximateDynamicSearch<'i, 's> {
    // pub const ALPHABET: &'static [u8] = "@ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    pub const ALPHABET: &'static [u8] = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes(); // TODO verander naar echt alfabet ^

    pub fn search(index: &BiFMIndex, pattern: String, dist: usize) -> Vec<u64> {
        if pattern.len() <= dist {
            panic!("Pattern length must be greater than edit distance");
        }

        // normal search when dist == 0
        if dist <= 0 {
            return index.normal_index.search_backward(pattern).locate();
        }

        let scheme = SearchScheme::get(dist);
        let search: ApproximateDynamicSearch = ApproximateDynamicSearch {
            index,
            pattern: pattern.as_str(),
            pattern_split: Vec::new(),
            scheme,
            forward: true,
            results: HashSet::new(),
        };

        search.perform_full_search()
    }

    fn register_result(&mut self, result: BiFMSearch) {
        result.locate().iter().for_each(|r| {
            self.results.insert(*r);
        });
    }

    fn perform_full_search(mut self) -> Vec<u64> {
        let part_amount = self.scheme.passes[0].upper.len();

        // split string
        let avg_length = self.pattern.len() as f64 / part_amount as f64;
        for i in 0..part_amount - 1 {
            self.pattern_split
                .push(&self.pattern[i * avg_length as usize..(i + 1) * avg_length as usize].as_ref());
        }
        self.pattern_split.push(&self.pattern[(part_amount - 1) * avg_length as usize..]);

        let mut exact_match_ranges: Vec<BiFMSearch> = Vec::new();
        self.forward = true;

        for part in &self.pattern_split {
            exact_match_ranges.push(self.index.search(*part));
        }

        let mut search_passes: Vec<PassDetails> = Vec::new();
        for pass in &self.scheme.passes {
            search_passes.push(self.make_pass(pass));
        }

        for pass in search_passes {
            self.perform_search_pass(pass, &exact_match_ranges);
            // println!("Results after pass: ");
            // let mut res: Vec<u64> = Vec::from_iter(self.results.clone());
            // res.sort();
            // for r in res {
            //     println!("{:#?}", r);
            // }
        }

        // filter redundant matches! searchscheme.h

        Vec::from_iter(self.results)
    }

    fn perform_search_pass(&mut self, pass: PassDetails, exact_match_ranges: &Vec<BiFMSearch>) {
        // get first part of search
        let s = &exact_match_ranges[pass.order[0] as usize];
        let mut search_status = BiFMSearch {
            index: s.index,
            backward_s: s.backward_s,
            backward_e: s.backward_e,
            forward_s: s.forward_s,
            forward_e: s.forward_e,
        };

        // discard useless pass
        if search_status.count() <= 0 {
            return;
        }

        let mut exact_matched_length = self.pattern_split[pass.order[0] as usize].len();
        let mut part_sequence_index = 1;

        // while no errors are allowed, keep doing exact matching
        while pass.upper[part_sequence_index] == 0 {
            let part_index = pass.order[part_sequence_index] as usize;
            let part = self.pattern_split[part_index];
            self.forward = pass.forward[part_sequence_index]; // ordered by pass step, not part index
            search_status = search_status.search(part, self.forward);
            if search_status.count() <= 0 {
                return;
            }
            exact_matched_length += part.len();
            part_sequence_index += 1;
        }

        // we now have a non-empty result collection and need to start approximate matching
        let start_occ = SearchDepthDist {
            search: search_status,
            dist: 0,
            depth: exact_matched_length as u32,
        };

        self.recursive_approx_match(&pass, start_occ, part_sequence_index);
    }

    fn recursive_approx_match(&mut self, pass: &PassDetails, start_occ: SearchDepthDist, part_sequence_index: usize) {
        // create banded matrix
        let mut matrix = BandedMatrix::new(
            self.pattern_split[pass.order[part_sequence_index] as usize].len() as i32,
            pass.upper[part_sequence_index] as i32 - start_occ.dist as i32,
            start_occ.dist as i32,
        );

        let mut stack: Vec<SearchDepthChar> = Vec::new();
        stack.reserve(self.pattern.len() * Self::ALPHABET.len());
        self.forward = pass.forward[part_sequence_index];

        self.extend_fm_pos(&start_occ.search, 0, &mut stack);

        while !stack.is_empty() {
            let current_pos = stack.pop().unwrap();

            let min_score = matrix.update_matrix_row(
                self.pattern_split[pass.order[part_sequence_index] as usize],
                current_pos.depth as i32,
                current_pos.char,
                self.forward,
            ) as u32;

            if min_score <= pass.upper[part_sequence_index] && current_pos.depth + 1 <= matrix.get_row_count() as u32 {
                self.extend_fm_pos(&current_pos.search, current_pos.depth, &mut stack);

                if matrix.in_final_col(current_pos.depth as i32) {
                    let last_col = matrix.get_final_col_value(current_pos.depth as i32) as u32;

                    if last_col <= pass.upper[part_sequence_index] && last_col >= pass.lower[part_sequence_index] {
                        if part_sequence_index == pass.order.len() - 1 {
                            self.register_result(current_pos.search);
                        } else {
                            self.recursive_approx_match(
                                pass,
                                SearchDepthDist {
                                    search: current_pos.search,
                                    depth: start_occ.depth + current_pos.depth,
                                    dist: last_col,
                                },
                                part_sequence_index + 1,
                            );
                            self.forward = pass.forward[part_sequence_index];
                        }
                    }
                }
            }
        }
    }

    fn extend_fm_pos<'a>(&mut self, search: &BiFMSearch<'a>, depth: u32, stack: &mut Vec<SearchDepthChar<'a>>) {
        for c in Self::ALPHABET {
            let base_search = search;
            let next_char_search = base_search.search_char(*c, self.forward);
            if next_char_search.count() > 0 {
                stack.push(SearchDepthChar { search: next_char_search, depth: depth + 1, char: *c })
            }
        }
    }

    fn make_pass(&self, pass: &SearchSchemePass) -> PassDetails {
        let mut forward: Vec<bool> = Vec::new();
        forward.push(true);
        for i in 1..pass.order.len() {
            forward.push(pass.order[i] > pass.order[i - 1])
        }

        PassDetails {
            order: pass.order.clone(),
            lower: pass.lower.clone(),
            upper: pass.upper.clone(),
            forward,
        }
    }
}
