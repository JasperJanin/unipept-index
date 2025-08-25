use crate::bd_index::BiFMIndex;
use crate::search::shared::SearchSchemePass;
use crate::search_scheme::SearchScheme;
use fm_index::BackwardSearchIndex;
use std::collections::HashSet;
use std::str;
use std::string::ToString;
use crate::search::search::BDFMSearch;

struct SearchStateEntry<'a> {
    search: BDFMSearch<'a>,
    errors: u8,
    chars_taken: usize,
    scheme_part_index: usize,
    forward: bool,
    remaining_string_slice: &'a str,
}

pub struct ApproximateStackSearch<'a> {
    index: &'a BiFMIndex,
    parts: &'a Vec<String>,
    pattern_length: usize,
    scheme_pass: &'a SearchSchemePass,
    results: HashSet<u64>,
    stack: Vec<SearchStateEntry<'a>>,
}

impl<'a> ApproximateStackSearch<'a> {
    // pub const ALPHABET: &'static [u8] = "ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    pub const ALPHABET: &'static [u8] = "@ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes(); // TODO verander naar echt alfabet ^

    pub fn search(index: &BiFMIndex, pattern: String, dist: usize) -> Vec<u64> {
        if pattern.len() <= dist {
            panic!("Pattern length must be greater than edit distance");
        }

        // normal search when dist == 0
        if dist <= 0 {
            return index.normal_index.search_backward(pattern).locate();
        }

        let scheme = SearchScheme::new(dist);

        let part_ct = scheme.passes[0].order.len();
        let chars_per_part = pattern.len() / part_ct;
        let mut parts = Vec::new();
        for i in 0..part_ct - 1 {
            parts.push(pattern[i * chars_per_part..(i + 1) * chars_per_part].to_string());
        }
        parts.push(pattern[(part_ct - 1) * chars_per_part..pattern.len()].to_string());

        let mut results = HashSet::new();

        for i in 0..scheme.pass_count {
            let mut search = ApproximateStackSearch {
                index,
                parts: &parts,
                pattern_length: pattern.len(),
                scheme_pass: &scheme.passes[i as usize],
                results: HashSet::new(),
                stack: Vec::new(),
            };

            search.stack.push(SearchStateEntry {
                search: BDFMSearch::new(&index),
                errors: 0,
                chars_taken: 0,
                scheme_part_index: 0,
                forward: false,
                remaining_string_slice: parts[scheme.passes[i as usize].order[0] as usize].as_str(),
            });

            while search.stack.len() > 0 {
                let state = search.stack.pop().unwrap();

                // check for total result
                if state.chars_taken >= search.pattern_length {
                    state.search.locate().iter().for_each(|r| {
                        search.results.insert(*r);
                    });
                    continue;
                }

                search.extend_search(state);
            }
            search.results.iter().for_each(|r| {
                results.insert(*r);
            });
        }
        Vec::from_iter(results)
    }

    fn extend_search(&mut self, mut state: SearchStateEntry<'a>) {
        if state.remaining_string_slice.len() == 0 {
            state.scheme_part_index += 1;
            state.remaining_string_slice =
                self.parts[self.scheme_pass.order[state.scheme_part_index] as usize].as_str();
            state.forward = self.scheme_pass.order[state.scheme_part_index] > self.scheme_pass.order[0];
        }

        let permitted_mistakes = self.scheme_pass.upper[state.scheme_part_index];

        let next_char = if state.forward {
            // todo move within errors allowed
            state.remaining_string_slice.bytes().next().unwrap()
        } else {
            state.remaining_string_slice.bytes().last().unwrap()
        };

        // errors allowed
        if state.errors < permitted_mistakes as u8 {
            for c in Self::ALPHABET {
                let mut search = BDFMSearch {
                    index: state.search.index,
                    backward_s: state.search.backward_s,
                    backward_e: state.search.backward_e,
                    forward_s: state.search.forward_s,
                    forward_e: state.search.forward_e,
                    // pattern: state.search.pattern.clone(),
                };
                search = search.search_char(*c, state.forward);

                if search.count() > 0 {
                    // match/mismatch
                    self.stack.push(SearchStateEntry {
                        search: BDFMSearch {
                            index: search.index,
                            backward_s: search.backward_s,
                            backward_e: search.backward_e,
                            forward_s: search.forward_s,
                            forward_e: search.forward_e,
                            // pattern: search.pattern.clone(),
                        },
                        errors: if *c == next_char { state.errors } else { state.errors + 1 },
                        chars_taken: state.chars_taken + 1,
                        scheme_part_index: state.scheme_part_index,
                        forward: state.forward,
                        remaining_string_slice: if state.forward {
                            &state.remaining_string_slice[1..]
                        } else {
                            &state.remaining_string_slice[..state.remaining_string_slice.len() - 1]
                        },
                    });

                    // insertion
                    self.stack.push(SearchStateEntry {
                        search,
                        errors: state.errors + 1,
                        chars_taken: state.chars_taken,
                        scheme_part_index: state.scheme_part_index,
                        forward: state.forward,
                        remaining_string_slice: state.remaining_string_slice,
                    });
                }
            }

            // deletion
            self.stack.push(SearchStateEntry {
                search: state.search,
                errors: state.errors + 1,
                chars_taken: state.chars_taken + 1,
                scheme_part_index: state.scheme_part_index,
                forward: state.forward,
                remaining_string_slice: if state.forward {
                    &state.remaining_string_slice[1..]
                } else {
                    &state.remaining_string_slice[..state.remaining_string_slice.len() - 1]
                },
            });
        }
        // errors not allowed
        else {
            let search = state.search.search(state.remaining_string_slice, state.forward);
            if search.count() > 0 {
                state.search = search;
                state.chars_taken += state.remaining_string_slice.len();
                state.remaining_string_slice = "";
                self.stack.push(state);
            }
        }
    }
}
