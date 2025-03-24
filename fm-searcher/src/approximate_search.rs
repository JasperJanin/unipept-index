use std::str;
use std::string::ToString;
use crate::bd_index::BidirectionalIndex;
use crate::search::BDFMSearch;

struct SearchStateEntry<'a> {
    search: BDFMSearch<'a>,
    errors: u8,
    chars_taken: usize,
    scheme_part_index: usize,
    forward: bool,
    remaining_string_slice: &'a str,
}


struct SearchSchemePass {
    order: Vec<u32>,
    lower: Vec<u32>,
    upper: Vec<u32>,
}

struct SearchScheme {
    runs: u32,
    passes: Vec<SearchSchemePass>
}

impl SearchScheme {
    pub fn new(dist: usize) -> Self {
        // TODO
        SearchScheme {
            runs: 2,
            passes: vec![
                SearchSchemePass {
                    order: vec![0, 1],
                    lower: vec![0, 0],
                    upper: vec![0, 1],
                },
                SearchSchemePass {
                    order: vec![1, 0],
                    lower: vec![0, 1],
                    upper: vec![0, 1],
                },
            ]
        }
    }
}

pub struct ApproximateSearch<'a> {
    index: &'a BidirectionalIndex,
    parts: &'a Vec<String>,
    pattern_length: usize,
    scheme_pass: &'a SearchSchemePass,
    results: Vec<u64>,
    stack: Vec<SearchStateEntry<'a>>,
    max_stack_size: usize,
}

impl<'a> ApproximateSearch<'a> {
    
    const ALPHABET: &'static [u8] = "ACDEFGHIKLMNOPQRSTUVWY".as_bytes();
    
    pub fn search(index: &BidirectionalIndex, pattern: String, dist: usize, max_stack_size: usize) -> Vec<u64> {
        // TODO handle dist too big or too small
    
        let part_ct = dist + 1;
        let chars_per_part = pattern.len() / part_ct;
        let mut parts = Vec::new();
        for i in 0..part_ct-1 {
            parts.push(pattern[i*chars_per_part .. (i+1)*chars_per_part].to_string());
        }
        parts.push(pattern[(part_ct-1) * chars_per_part .. pattern.len()].to_string());
        
        let scheme = SearchScheme::new(dist);
        
        let mut results = Vec::new();
        
        for i in 0..scheme.runs {
            let mut stack = Vec::new();
            stack.push(SearchStateEntry {
                search: BDFMSearch::new(&index),
                errors: 0,
                chars_taken: 0,
                scheme_part_index: 0,
                forward: false,
                remaining_string_slice: parts[scheme.passes[i as usize].order[0] as usize].as_str()
            });
    
            let mut search = ApproximateSearch {
                index,
                parts: &parts,
                pattern_length: pattern.len(),
                scheme_pass: &scheme.passes[i as usize],
                results: Vec::new(),
                stack,
                max_stack_size,
            };
    
            while search.stack.len() > 0 {
                let state = search.stack.pop().unwrap();
    
                // check for total result
                if state.chars_taken >= search.pattern_length {
                    search.results.append(&mut state.search.locate());
                    continue;
                }
    
                search.extend_search(state);
            }
            results.append(search.results.as_mut());
        }

        results
    }
    
    fn extend_search(&mut self, mut state: SearchStateEntry<'a>) {
        
        let scheme_part_index = state.chars_taken / (self.pattern_length / self.parts.len());
        
        let permitted_mistakes = self.scheme_pass.upper[scheme_part_index];
        
        if state.remaining_string_slice.len() == 0 {
            state.remaining_string_slice = self.parts[self.scheme_pass.order[scheme_part_index] as usize].as_str();
            state.forward = self.scheme_pass.order[scheme_part_index] > self.scheme_pass.order[0];
        }
        
        
        let next_char = if state.forward {
            state.remaining_string_slice.bytes().last().unwrap()
        } else {
            state.remaining_string_slice.bytes().next().unwrap()
        };
        // 
        //     // errors allowed
        if state.errors < permitted_mistakes as u8 {
                for c in Self::ALPHABET { // todo check of `let search = state.search` kan
                    let search = BDFMSearch {
                        index: state.search.index,
                        backward_s: state.search.backward_s,
                        backward_e: state.search.backward_e,
                        forward_s: state.search.forward_s,
                        forward_e: state.search.forward_e,
                        pattern: state.search.pattern.clone()
                    };
                    let search = search.search_char(*c, state.forward);
                    
                    if search.count() > 0 {
                        // match/mismatch
                        self.stack.push(SearchStateEntry {
                            search: BDFMSearch {
                                index: state.search.index,
                                backward_s: state.search.backward_s,
                                backward_e: state.search.backward_e,
                                forward_s: state.search.forward_s,
                                forward_e: state.search.forward_e,
                                pattern: search.pattern.clone(),
                            },
                            errors: if *c == next_char { state.errors } else { state.errors + 1 },
                            chars_taken: state.chars_taken + 1,
                            scheme_part_index: state.scheme_part_index,
                            forward: state.forward,
                            remaining_string_slice: if !state.forward { &state.remaining_string_slice[1..] } else { &state.remaining_string_slice[..state.remaining_string_slice.len()-1] },
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
                }}
                
                // deletion
                let search = state.search.search_char(next_char, state.forward);
                self.stack.push(SearchStateEntry {
                    search,
                    errors: state.errors + 1,
                    chars_taken: state.chars_taken + 1,
                    scheme_part_index,
                    forward: false,
                    remaining_string_slice: "",
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

pub fn searchscheme_test() -> Vec<u64> {

    let index = BidirectionalIndex::load();
    
    let search_term = "CBB".to_string();
    
    let results = index.find_approximate_matches(search_term, 1);
    results
}
